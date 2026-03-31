use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::container::spoof as container_spoof;
use crate::spoof::HwIdentity;
use crate::spoof::generate_identity;

pub mod deps;
pub mod display;
pub mod exec;
pub mod session;
pub mod spoof;
pub mod steam_auth;
pub mod steam_library;
pub mod update;
pub mod verify;

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("docker command failed: {0}")]
    Docker(String),
    #[error("spoof error: {0}")]
    Spoof(#[from] container_spoof::ContainerSpoofError),
}

#[derive(Debug, Clone)]
pub struct ContainerConfig {
    pub name: String,
    pub image: String,
    pub memory_limit: String,
    pub cpu_limit: String,
    pub vnc_port: u16,
    pub hw: HwIdentity,
    pub cs2_shared_dir: Option<String>,
    pub spoof_dir: String,
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ContainerState {
    Running,
    Exited,
    Created,
    Paused,
    Restarting,
    Dead,
    Removing,
    #[serde(untagged)]
    Unknown(String),
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerState::Running => write!(f, "running"),
            ContainerState::Exited => write!(f, "exited"),
            ContainerState::Created => write!(f, "created"),
            ContainerState::Paused => write!(f, "paused"),
            ContainerState::Restarting => write!(f, "restarting"),
            ContainerState::Dead => write!(f, "dead"),
            ContainerState::Removing => write!(f, "removing"),
            ContainerState::Unknown(s) => write!(f, "{s}"),
        }
    }
}

impl From<&str> for ContainerState {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" => ContainerState::Running,
            "exited" => ContainerState::Exited,
            "created" => ContainerState::Created,
            "paused" => ContainerState::Paused,
            "restarting" => ContainerState::Restarting,
            "dead" => ContainerState::Dead,
            "removing" => ContainerState::Removing,
            other => ContainerState::Unknown(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub state: ContainerState,
    pub image: String,
    pub vnc_port: Option<u16>,
}

fn docker(args: &[&str]) -> Result<String, ContainerError> {
    let output = Command::new("docker")
        .args(args)
        .output()
        .map_err(|e| ContainerError::Docker(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(ContainerError::Docker(stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn create(config: &ContainerConfig) -> Result<String, ContainerError> {
    let spoof_files =
        container_spoof::create_spoof_files(&config.spoof_dir, &config.name, &config.hw)?;

    let mut args: Vec<String> = vec![
        "run".into(),
        "-d".into(),
        "--name".into(),
        config.name.clone(),
        "--hostname".into(),
        config.name.clone(),
    ];

    args.push("--memory".into());
    args.push(config.memory_limit.clone());
    args.push("--cpus".into());
    args.push(config.cpu_limit.clone());

    args.push("--device".into());
    args.push("/dev/dri:/dev/dri".into());

    args.extend(container_spoof::mac_docker_arg(&config.hw.mac_address));
    args.extend(spoof_files.docker_args);

    args.push("-p".into());
    args.push(format!("{}:5900", config.vnc_port));

    if let Some(ref cs2_dir) = config.cs2_shared_dir {
        args.push("-v".into());
        args.push(format!("{cs2_dir}:/opt/cs2:ro"));
    }

    args.push("--cap-add".into());
    args.push("SYS_ADMIN".into());
    args.push("--cap-add".into());
    args.push("SYS_NICE".into());
    args.push("--ipc".into());
    args.push("host".into());

    args.extend(config.extra_args.clone());
    args.push(config.image.clone());

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = docker(&args_ref)?;

    Ok(output.trim().to_string())
}

pub fn start(name: &str) -> Result<String, ContainerError> {
    docker(&["start", name])
}

pub fn stop(name: &str) -> Result<String, ContainerError> {
    docker(&["stop", "-t", "10", name])
}

pub fn kill(name: &str) -> Result<String, ContainerError> {
    docker(&["kill", name])
}

pub fn remove(name: &str, force: bool) -> Result<String, ContainerError> {
    if force {
        docker(&["rm", "-f", name])
    } else {
        docker(&["rm", name])
    }
}

pub fn state(name: &str) -> Result<ContainerState, ContainerError> {
    let output = docker(&["inspect", "-f", "{{.State.Status}}", name])?;
    Ok(ContainerState::from(output.trim()))
}

pub fn list_all(prefix: &str) -> Result<Vec<ContainerInfo>, ContainerError> {
    let format_str = "{{.Names}}\t{{.State}}\t{{.Image}}\t{{.Ports}}";
    let filter = format!("name=^{prefix}");
    let output = docker(&["ps", "-a", "--format", format_str, "--filter", &filter])?;

    let mut containers = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 4 {
            continue;
        }

        let vnc_port = extract_vnc_port(parts[3]);

        containers.push(ContainerInfo {
            name: parts[0].to_string(),
            state: ContainerState::from(parts[1]),
            image: parts[2].to_string(),
            vnc_port,
        });
    }

    Ok(containers)
}

fn extract_vnc_port(ports_str: &str) -> Option<u16> {
    for mapping in ports_str.split(',') {
        let mapping = mapping.trim();
        if mapping.contains("5900")
            && let Some(host_part) = mapping.split("->").next()
            && let Some(port_str) = host_part.rsplit(':').next()
            && let Ok(port) = port_str.parse::<u16>()
        {
            return Some(port);
        }
    }
    None
}

pub fn default_config(
    name: &str,
    image: &str,
    vnc_port: u16,
    cs2_shared_dir: Option<&str>,
) -> ContainerConfig {
    let hw = generate_identity(name);
    ContainerConfig {
        name: name.to_string(),
        image: image.to_string(),
        memory_limit: "2g".to_string(),
        cpu_limit: "2.0".to_string(),
        vnc_port,
        hw,
        cs2_shared_dir: cs2_shared_dir.map(String::from),
        spoof_dir: "/var/lib/vmctl/container-spoof".to_string(),
        extra_args: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_container_state_display() {
        assert_eq!(ContainerState::Running.to_string(), "running");
        assert_eq!(ContainerState::Exited.to_string(), "exited");
        assert_eq!(
            ContainerState::Unknown("custom".into()).to_string(),
            "custom"
        );
    }

    #[test]
    fn test_container_state_from_str() {
        assert_eq!(ContainerState::from("running"), ContainerState::Running);
        assert_eq!(ContainerState::from("RUNNING"), ContainerState::Running);
        assert_eq!(ContainerState::from("exited"), ContainerState::Exited);
        assert_eq!(ContainerState::from("created"), ContainerState::Created);
        assert_eq!(ContainerState::from("paused"), ContainerState::Paused);
        assert_eq!(
            ContainerState::from("wat"),
            ContainerState::Unknown("wat".into())
        );
    }

    #[test]
    fn test_extract_vnc_port() {
        assert_eq!(extract_vnc_port("0.0.0.0:5901->5900/tcp"), Some(5901));
        assert_eq!(
            extract_vnc_port("0.0.0.0:5901->5900/tcp, :::5901->5900/tcp"),
            Some(5901)
        );
        assert_eq!(extract_vnc_port(""), None);
        assert_eq!(extract_vnc_port("0.0.0.0:8080->80/tcp"), None);
    }

    #[test]
    fn test_default_config() {
        let cfg = default_config("cs2-farm-0", "cs2-farm:latest", 5901, Some("/opt/cs2-shared"));
        assert_eq!(cfg.name, "cs2-farm-0");
        assert_eq!(cfg.image, "cs2-farm:latest");
        assert_eq!(cfg.vnc_port, 5901);
        assert_eq!(cfg.memory_limit, "2g");
        assert_eq!(cfg.cpu_limit, "2.0");
        assert_eq!(cfg.cs2_shared_dir.as_deref(), Some("/opt/cs2-shared"));
        assert!(!cfg.hw.mac_address.is_empty());
    }

    #[test]
    fn test_container_info_serialization() {
        let info = ContainerInfo {
            name: "cs2-farm-0".into(),
            state: ContainerState::Running,
            image: "cs2-farm:latest".into(),
            vnc_port: Some(5901),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"name\":\"cs2-farm-0\""));
        assert!(json.contains("\"state\":\"running\""));
        assert!(json.contains("\"vnc_port\":5901"));
    }

    #[test]
    fn test_container_state_serialization() {
        let state = ContainerState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"running\"");

        let state = ContainerState::Unknown("custom".into());
        let json = serde_json::to_string(&state).unwrap();
        assert!(json.contains("custom"));
    }
}
