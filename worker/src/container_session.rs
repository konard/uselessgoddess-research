//! Steam session injection for containers.
//!
//! This is the container equivalent of `session.rs`. Instead of using the
//! QEMU Guest Agent, it uses `docker exec` to write session files into
//! the container.

use crate::container_exec;
use crate::session::SteamSession;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContainerSessionError {
    #[error("container exec error: {0}")]
    Exec(#[from] container_exec::ContainerExecError),
    #[error("session injection failed for container `{container}`: {reason}")]
    Injection { container: String, reason: String },
}

/// Default path to Steam config directory inside the container.
const STEAM_CONFIG_DIR: &str = "/home/farmuser/.steam/steam/config";

/// Inject a Steam session into a running container.
///
/// Writes config.vdf, loginusers.vdf, and .ready marker via `docker exec`.
/// This is the container equivalent of `session::inject_session`.
pub fn inject_session(
    container_name: &str,
    session: &SteamSession,
    config_dir: Option<&str>,
) -> Result<(), ContainerSessionError> {
    let dir = config_dir.unwrap_or(STEAM_CONFIG_DIR);

    // Ensure config directory exists
    container_exec::exec(container_name, &format!("mkdir -p '{dir}'"))?;

    // Write config.vdf
    let config_vdf = crate::session::generate_config_vdf(session);
    container_exec::write_file(
        container_name,
        &format!("{dir}/config.vdf"),
        config_vdf.as_bytes(),
    )?;

    // Write loginusers.vdf
    let loginusers_vdf = crate::session::generate_loginusers_vdf(session);
    container_exec::write_file(
        container_name,
        &format!("{dir}/loginusers.vdf"),
        loginusers_vdf.as_bytes(),
    )?;

    // Set ownership (files written by root via docker exec need to be owned by farmuser)
    container_exec::exec(
        container_name,
        &format!("chown -R farmuser:farmuser '{dir}'"),
    )?;

    // Write .ready marker (triggers steam-launcher.sh)
    container_exec::write_file(container_name, &format!("{dir}/.ready"), b"1")?;

    Ok(())
}

/// Switch a container to a different Steam account.
///
/// 1. Kills Steam process inside container
/// 2. Injects new session files
/// 3. The steam-launcher.sh loop will pick up the new .ready marker
pub fn switch_account(
    container_name: &str,
    session: &SteamSession,
    config_dir: Option<&str>,
) -> Result<(), ContainerSessionError> {
    // Kill existing Steam process
    let kill_result = container_exec::exec(container_name, "pkill -TERM -f steam || true");
    if let Err(e) = kill_result {
        eprintln!("Warning: could not kill Steam in container '{container_name}': {e}");
    }

    // Wait for Steam to exit
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Inject new session
    inject_session(container_name, session, config_dir)?;

    Ok(())
}

/// Check if Steam is running inside a container.
pub fn is_steam_running(container_name: &str) -> Result<bool, ContainerSessionError> {
    let result = container_exec::exec(container_name, "pgrep -c steam 2>/dev/null || echo 0")?;
    let count: i32 = result.stdout.trim().parse().unwrap_or(0);
    Ok(count > 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session() -> SteamSession {
        SteamSession {
            account_name: "testuser123".into(),
            refresh_token: "eyJhbGciOiJFZERTQSJ9.test_token_data".into(),
            steam_id: "76561198012345678".into(),
            persona_name: "TestPlayer".into(),
        }
    }

    #[test]
    fn test_sample_session_is_valid() {
        let session = sample_session();
        assert!(!session.account_name.is_empty());
        assert!(!session.refresh_token.is_empty());
        assert!(!session.steam_id.is_empty());
    }

    #[test]
    fn test_default_config_dir() {
        assert_eq!(STEAM_CONFIG_DIR, "/home/farmuser/.steam/steam/config");
    }
}
