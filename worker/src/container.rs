//! Docker container lifecycle management for CS2 farming.
//!
//! This module is the container equivalent of `vm.rs` + `config.rs`.
//! It manages the full lifecycle of CS2 farming containers:
//! - Create (docker run with GPU, spoofing, CS2 shared mount)
//! - Start / Stop / Destroy / Remove
//! - List running containers
//!
//! Each container gets:
//! - Access to GPU via /dev/dri (AMD/Intel render nodes)
//! - Spoofed hardware identity (MAC, machine-id, DMI files)
//! - Shared CS2 installation via bind mount
//! - Wayland (Sway) compositor + wayvnc for display
//! - Unique hostname and container name

use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::container_spoof;
use crate::spoof::{self, HwIdentity};

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("docker command failed: {0}")]
    Docker(String),
    #[error("container `{0}` not found")]
    NotFound(String),
    #[error("container `{0}` already exists")]
    AlreadyExists(String),
    #[error("spoof error: {0}")]
    Spoof(#[from] container_spoof::ContainerSpoofError),
    #[error("I/O error: {0}")]
    Io(String),
}

/// Container configuration for creation.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Container name (also used as seed for hardware spoofing).
    pub name: String,
    /// Docker image to use.
    pub image: String,
    /// RAM limit (e.g., "2g").
    pub memory_limit: String,
    /// CPU limit (e.g., "2.0" for 2 cores).
    pub cpu_limit: String,
    /// VNC port to expose on host.
    pub vnc_port: u16,
    /// Hardware identity for spoofing.
    pub hw: HwIdentity,
    /// Host path for shared CS2 directory.
    pub cs2_shared_dir: Option<String>,
    /// Host directory to store spoof files.
    pub spoof_dir: String,
    /// Additional Docker flags (e.g., for custom networks).
    pub extra_args: Vec<String>,
}

/// State of a container.
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

/// Info about a container.
#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub name: String,
    pub state: ContainerState,
    pub image: String,
    pub vnc_port: Option<u16>,
}

/// Run a docker command and return stdout on success.
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

/// Create and start a new container with GPU access, spoofing, and CS2 mount.
///
/// This is the container equivalent of `cmd_create` + `vm::define` + `vm::start`.
pub fn create(config: &ContainerConfig) -> Result<String, ContainerError> {
    // Generate spoof files on host
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

    // Resource limits
    args.push("--memory".into());
    args.push(config.memory_limit.clone());
    args.push("--cpus".into());
    args.push(config.cpu_limit.clone());

    // GPU access: pass through entire /dev/dri for AMD/Intel render nodes
    args.push("--device".into());
    args.push("/dev/dri:/dev/dri".into());

    // MAC address spoofing
    args.extend(container_spoof::mac_docker_arg(&config.hw.mac_address));

    // Hardware spoofing bind mounts (DMI, machine-id)
    args.extend(spoof_files.docker_args);

    // VNC port mapping
    args.push("-p".into());
    args.push(format!("{}:5900", config.vnc_port));

    // CS2 shared directory (read-only bind mount)
    if let Some(ref cs2_dir) = config.cs2_shared_dir {
        args.push("-v".into());
        args.push(format!("{cs2_dir}:/opt/cs2:ro"));
    }

    // Security: needed for /sys bind mounts and network config
    args.push("--cap-add".into());
    args.push("SYS_ADMIN".into());
    // Needed for sway/wlroots
    args.push("--cap-add".into());
    args.push("SYS_NICE".into());

    // IPC for shared memory (needed by Wayland/Vulkan)
    args.push("--ipc".into());
    args.push("host".into());

    // Extra user-provided arguments
    args.extend(config.extra_args.clone());

    // Image
    args.push(config.image.clone());

    let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = docker(&args_ref)?;

    Ok(output.trim().to_string())
}

/// Start a stopped container.
pub fn start(name: &str) -> Result<String, ContainerError> {
    docker(&["start", name])
}

/// Stop a running container (graceful).
pub fn stop(name: &str) -> Result<String, ContainerError> {
    docker(&["stop", "-t", "10", name])
}

/// Force-kill a container.
pub fn kill(name: &str) -> Result<String, ContainerError> {
    docker(&["kill", name])
}

/// Remove a container (must be stopped first, or use force=true).
pub fn remove(name: &str, force: bool) -> Result<String, ContainerError> {
    if force {
        docker(&["rm", "-f", name])
    } else {
        docker(&["rm", name])
    }
}

/// Get the state of a container.
pub fn state(name: &str) -> Result<ContainerState, ContainerError> {
    let output = docker(&["inspect", "-f", "{{.State.Status}}", name])?;
    Ok(ContainerState::from(output.trim()))
}

/// List all CS2 farm containers.
///
/// Filters containers by the "cs2-farm" label or name prefix.
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

/// Extract the host VNC port from Docker's port mapping string.
///
/// Input format: "0.0.0.0:5901->5900/tcp" → Some(5901)
fn extract_vnc_port(ports_str: &str) -> Option<u16> {
    for mapping in ports_str.split(',') {
        let mapping = mapping.trim();
        if mapping.contains("5900") {
            // Format: "0.0.0.0:5901->5900/tcp" or ":::5901->5900/tcp"
            if let Some(host_part) = mapping.split("->").next()
                && let Some(port_str) = host_part.rsplit(':').next()
                && let Ok(port) = port_str.parse::<u16>()
            {
                return Some(port);
            }
        }
    }
    None
}

/// Create a ContainerConfig with sensible defaults.
pub fn default_config(
    name: &str,
    image: &str,
    vnc_port: u16,
    cs2_shared_dir: Option<&str>,
) -> ContainerConfig {
    let hw = spoof::generate_identity(name);
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
        assert_eq!(
            extract_vnc_port("0.0.0.0:5901->5900/tcp"),
            Some(5901)
        );
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
