//! CS2 update management across VMs.
//!
//! Implements the centralized update strategy using virtiofs:
//! - One shared CS2 installation on the host (`/opt/cs2-shared/`)
//! - All VMs mount it read-only via virtiofs
//! - Updates are performed by stopping VMs, running steamcmd on host,
//!   then restarting VMs
//!
//! The update coordinator uses a lock file to prevent concurrent updates.

use std::path::Path;
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::guest_agent;

#[derive(Debug, Error)]
pub enum UpdateError {
    #[error("update already in progress (lock: {0})")]
    LockExists(String),
    #[error("shared directory not found: {0}")]
    SharedDirNotFound(String),
    #[error("steamcmd failed: {0}")]
    SteamCmd(String),
    #[error("I/O error: {0}")]
    Io(String),
    #[error("guest agent error: {0}")]
    GuestAgent(String),
}

impl From<guest_agent::GuestAgentError> for UpdateError {
    fn from(e: guest_agent::GuestAgentError) -> Self {
        UpdateError::GuestAgent(e.to_string())
    }
}

/// CS2 update configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Host path to the shared CS2 installation.
    pub shared_dir: String,
    /// Path to the lock file during updates.
    pub lock_file: String,
    /// Steam login for steamcmd (anonymous for CS2).
    pub steam_login: String,
    /// CS2 app ID.
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

/// Status of the CS2 installation.
#[derive(Debug, Clone, Serialize)]
pub struct Cs2Status {
    pub installed: bool,
    pub shared_dir: String,
    pub update_in_progress: bool,
    pub manifest_exists: bool,
}

/// Check the current status of the shared CS2 installation.
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

/// Acquire the update lock. Returns error if another update is in progress.
pub fn acquire_lock(config: &UpdateConfig) -> Result<(), UpdateError> {
    if Path::new(&config.lock_file).exists() {
        return Err(UpdateError::LockExists(config.lock_file.clone()));
    }
    let content = format!(
        "pid={}\nstarted={}",
        std::process::id(),
        chrono_now_fallback()
    );
    std::fs::write(&config.lock_file, content).map_err(|e| UpdateError::Io(e.to_string()))?;
    Ok(())
}

/// Release the update lock.
pub fn release_lock(config: &UpdateConfig) -> Result<(), UpdateError> {
    if Path::new(&config.lock_file).exists() {
        std::fs::remove_file(&config.lock_file).map_err(|e| UpdateError::Io(e.to_string()))?;
    }
    Ok(())
}

/// Run steamcmd to update CS2 in the shared directory.
///
/// This should be called after all VMs have been stopped or have
/// unmounted the shared directory.
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

/// Notify a running VM to restart CS2 after an update.
///
/// Uses guest agent to kill the CS2 process and let the systemd
/// service restart it automatically.
pub fn notify_vm_restart_cs2(vm_name: &str) -> Result<(), UpdateError> {
    // Kill CS2 process; systemd will auto-restart it
    guest_agent::exec(vm_name, "pkill -TERM -f cs2 || true")?;
    Ok(())
}

/// Perform a full update cycle:
/// 1. Acquire lock
/// 2. Notify VMs to stop CS2
/// 3. Run steamcmd update
/// 4. Release lock
/// 5. Notify VMs to restart CS2
pub fn perform_update(
    config: &UpdateConfig,
    vm_names: &[String],
) -> Result<String, UpdateError> {
    // Step 1: Lock
    acquire_lock(config)?;

    // Step 2: Stop CS2 in all VMs
    for vm in vm_names {
        if let Err(e) = notify_vm_restart_cs2(vm) {
            eprintln!("Warning: could not stop CS2 in VM '{vm}': {e}");
        }
    }

    // Step 3: Update
    let result = run_steamcmd_update(config);

    // Step 4: Always release lock
    let _ = release_lock(config);

    let output = result?;

    // Step 5: VMs will auto-restart CS2 via systemd

    Ok(output)
}

/// Simple timestamp without chrono dependency.
fn chrono_now_fallback() -> String {
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

        // Clean up from previous runs
        let _ = std::fs::remove_file(&lock_path);

        let cfg = UpdateConfig {
            lock_file: lock_path.clone(),
            ..Default::default()
        };

        // Acquire should succeed
        assert!(acquire_lock(&cfg).is_ok());
        assert!(Path::new(&lock_path).exists());

        // Double acquire should fail
        assert!(matches!(
            acquire_lock(&cfg),
            Err(UpdateError::LockExists(_))
        ));

        // Release should succeed
        assert!(release_lock(&cfg).is_ok());
        assert!(!Path::new(&lock_path).exists());

        // Release on non-existent is ok
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
