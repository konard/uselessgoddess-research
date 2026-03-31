use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::container::exec as container_exec;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("update already in progress (lock: {0})")]
    LockExists(String),
    #[error("steamcmd failed: {0}")]
    SteamCmd(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("container exec error: {0}")]
    ContainerExec(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub shared_dir: String,
    pub lock_file: String,
    pub steam_login: String,
    pub app_id: u32,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            shared_dir: "/opt/cs2-shared".into(),
            lock_file: "/opt/cs2-shared/.update.lock".into(),
            steam_login: "anonymous".into(),
            app_id: 730,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Cs2Status {
    pub installed: bool,
    pub shared_dir: String,
    pub update_in_progress: bool,
    pub manifest_exists: bool,
}

pub fn check_status(config: &UpdateConfig) -> Cs2Status {
    let shared_exists = Path::new(&config.shared_dir).is_dir();
    let lock_exists = Path::new(&config.lock_file).exists();
    let manifest = format!(
        "{}/steamapps/appmanifest_{}.acf",
        config.shared_dir, config.app_id
    );
    let manifest_exists = Path::new(&manifest).exists();

    Cs2Status {
        installed: shared_exists && manifest_exists,
        shared_dir: config.shared_dir.clone(),
        update_in_progress: lock_exists,
        manifest_exists,
    }
}

pub fn acquire_lock(config: &UpdateConfig) -> Result<(), UpdateError> {
    if Path::new(&config.lock_file).exists() {
        return Err(UpdateError::LockExists(config.lock_file.clone()));
    }
    let content = format!(
        "pid={}\nstarted={}",
        std::process::id(),
        timestamp_now()
    );
    std::fs::write(&config.lock_file, content).map_err(|e| UpdateError::Io(e.to_string()))?;
    Ok(())
}

pub fn release_lock(config: &UpdateConfig) -> Result<(), UpdateError> {
    if Path::new(&config.lock_file).exists() {
        std::fs::remove_file(&config.lock_file).map_err(|e| UpdateError::Io(e.to_string()))?;
    }
    Ok(())
}

pub fn run_steamcmd_update(config: &UpdateConfig) -> Result<String, UpdateError> {
    if !Path::new(&config.shared_dir).is_dir() {
        std::fs::create_dir_all(&config.shared_dir)
            .map_err(|e| UpdateError::Io(e.to_string()))?;
    }

    let output = Command::new("steamcmd")
        .args([
            "+force_install_dir",
            &config.shared_dir,
            "+login",
            &config.steam_login,
            &format!("+app_update {} validate", config.app_id),
            "+quit",
        ])
        .output()
        .map_err(|e| UpdateError::SteamCmd(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        return Err(UpdateError::SteamCmd(format!(
            "exit code: {:?}\nstderr: {stderr}",
            output.status.code()
        )));
    }

    Ok(stdout)
}

fn notify_container_restart_cs2(container_name: &str) -> Result<(), UpdateError> {
    container_exec::exec(container_name, "pkill -TERM -f cs2 || true")
        .map_err(|e| UpdateError::ContainerExec(e.to_string()))?;
    Ok(())
}

pub fn perform_update(
    config: &UpdateConfig,
    container_names: &[String],
) -> Result<String, UpdateError> {
    acquire_lock(config)?;

    for name in container_names {
        if let Err(e) = notify_container_restart_cs2(name) {
            eprintln!("Warning: could not stop CS2 in container '{name}': {e}");
        }
    }

    let result = run_steamcmd_update(config);

    let _ = release_lock(config);

    result
}

fn timestamp_now() -> String {
    let output = Command::new("date")
        .arg("+%Y-%m-%dT%H:%M:%S%z")
        .output();
    match output {
        Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        Err(_) => "unknown".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_config_default() {
        let cfg = UpdateConfig::default();
        assert_eq!(cfg.shared_dir, "/opt/cs2-shared");
        assert_eq!(cfg.app_id, 730);
        assert_eq!(cfg.steam_login, "anonymous");
    }

    #[test]
    fn test_check_status_nonexistent() {
        let cfg = UpdateConfig {
            shared_dir: "/nonexistent/cs2-shared".into(),
            lock_file: "/nonexistent/.lock".into(),
            ..Default::default()
        };
        let status = check_status(&cfg);
        assert!(!status.installed);
        assert!(!status.update_in_progress);
        assert!(!status.manifest_exists);
    }

    #[test]
    fn test_cs2_status_serialization() {
        let status = Cs2Status {
            installed: true,
            shared_dir: "/opt/cs2".into(),
            update_in_progress: false,
            manifest_exists: true,
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"installed\":true"));
        assert!(json.contains("\"update_in_progress\":false"));
    }

    #[test]
    fn test_lock_lifecycle() {
        let tmp = std::env::temp_dir().join("vmctl-test-lock");
        let lock_path = tmp.to_str().unwrap().to_string();

        let _ = std::fs::remove_file(&lock_path);

        let cfg = UpdateConfig {
            lock_file: lock_path.clone(),
            ..Default::default()
        };

        assert!(acquire_lock(&cfg).is_ok());
        assert!(Path::new(&lock_path).exists());

        assert!(matches!(
            acquire_lock(&cfg),
            Err(UpdateError::LockExists(_))
        ));

        assert!(release_lock(&cfg).is_ok());
        assert!(!Path::new(&lock_path).exists());

        assert!(release_lock(&cfg).is_ok());
    }

    #[test]
    fn test_update_config_serialization() {
        let cfg = UpdateConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: UpdateConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.app_id, 730);
        assert_eq!(parsed.shared_dir, "/opt/cs2-shared");
    }
}
