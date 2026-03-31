//! CS2 update management for containers.
//!
//! Container equivalent of `update.rs`. The update strategy is the same:
//! CS2 is shared via a bind mount from the host. Updates are performed
//! on the host via steamcmd, then containers see the new files immediately.
//!
//! The only difference is that we use `docker exec` instead of the QEMU
//! Guest Agent to notify containers to restart CS2.

use crate::container_exec;
use crate::update::{self, UpdateConfig, UpdateError};

/// Notify a running container to restart CS2 after an update.
///
/// Uses `docker exec` to kill the CS2 process. The steam-launcher.sh
/// script inside the container will need to be re-triggered.
pub fn notify_container_restart_cs2(container_name: &str) -> Result<(), UpdateError> {
    container_exec::exec(container_name, "pkill -TERM -f cs2 || true")
        .map_err(|e| UpdateError::GuestAgent(e.to_string()))?;
    Ok(())
}

/// Perform a full update cycle for containers:
/// 1. Acquire lock
/// 2. Notify containers to stop CS2
/// 3. Run steamcmd update on host
/// 4. Release lock
/// 5. Containers auto-restart CS2 via steam-launcher loop
pub fn perform_update(
    config: &UpdateConfig,
    container_names: &[String],
) -> Result<String, UpdateError> {
    // Step 1: Lock
    update::acquire_lock(config)?;

    // Step 2: Stop CS2 in all containers
    for name in container_names {
        if let Err(e) = notify_container_restart_cs2(name) {
            eprintln!("Warning: could not stop CS2 in container '{name}': {e}");
        }
    }

    // Step 3: Update
    let result = update::run_steamcmd_update(config);

    // Step 4: Always release lock
    let _ = update::release_lock(config);

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_config_default_works() {
        let cfg = UpdateConfig::default();
        assert_eq!(cfg.shared_dir, "/opt/cs2-shared");
        assert_eq!(cfg.app_id, 730);
    }
}
