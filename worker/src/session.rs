//! Steam session injection for automatic login.
//!
//! Injects Steam session files (config.vdf, loginusers.vdf) into a VM
//! via the QEMU Guest Agent, enabling automatic Steam login without
//! user interaction.
//!
//! Two approaches are supported:
//! 1. **Refresh token injection** (preferred): Write config.vdf with a
//!    pre-obtained refresh token. Steam auto-logs in on launch.
//! 2. **Session file copy**: Copy a complete set of Steam config files
//!    from a reference machine.

use serde::{Deserialize, Serialize};

use crate::guest_agent;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("guest agent error: {0}")]
    GuestAgent(#[from] guest_agent::GuestAgentError),
    #[error("session injection failed for VM `{vm}`: {reason}")]
    Injection { vm: String, reason: String },
}

/// Steam account session data for injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamSession {
    /// Steam account name.
    pub account_name: String,
    /// Steam refresh token (long-lived, ~200 days).
    pub refresh_token: String,
    /// Steam ID (64-bit).
    pub steam_id: String,
    /// Optional persona name for display.
    pub persona_name: String,
}

/// Default path to Steam config directory inside the guest.
const STEAM_CONFIG_DIR: &str = "/home/farmuser/.steam/steam/config";

/// Generate config.vdf content with refresh token for auto-login.
pub fn generate_config_vdf(session: &SteamSession) -> String {
    format!(
        r#""InstallConfigStore"
{{
	"Software"
	{{
		"Valve"
		{{
			"Steam"
			{{
				"AutoLoginUser"		"{account}"
				"Accounts"
				{{
					"{account}"
					{{
						"SteamID"		"{steam_id}"
					}}
				}}
				"ConnectCache"
				{{
					"{account}"
					{{
						"Token"		"{token}"
					}}
				}}
			}}
		}}
	}}
}}"#,
        account = session.account_name,
        steam_id = session.steam_id,
        token = session.refresh_token,
    )
}

/// Generate loginusers.vdf content for auto-login.
pub fn generate_loginusers_vdf(session: &SteamSession) -> String {
    format!(
        r#""users"
{{
	"{steam_id}"
	{{
		"AccountName"		"{account}"
		"PersonaName"		"{persona}"
		"RememberPassword"		"1"
		"AllowAutoLogin"		"1"
		"MostRecent"		"1"
	}}
}}"#,
        steam_id = session.steam_id,
        account = session.account_name,
        persona = session.persona_name,
    )
}

/// Inject a Steam session into a running VM via the guest agent.
///
/// This writes the config.vdf and loginusers.vdf files, then creates
/// a `.ready` marker file that the steam-launcher service watches for.
pub fn inject_session(
    vm_name: &str,
    session: &SteamSession,
    config_dir: Option<&str>,
) -> Result<(), SessionError> {
    let dir = config_dir.unwrap_or(STEAM_CONFIG_DIR);

    // Ensure config directory exists
    guest_agent::exec(vm_name, &format!("mkdir -p {dir}"))?;

    // Write config.vdf
    let config_vdf = generate_config_vdf(session);
    guest_agent::write_file(
        vm_name,
        &format!("{dir}/config.vdf"),
        config_vdf.as_bytes(),
    )?;

    // Write loginusers.vdf
    let loginusers_vdf = generate_loginusers_vdf(session);
    guest_agent::write_file(
        vm_name,
        &format!("{dir}/loginusers.vdf"),
        loginusers_vdf.as_bytes(),
    )?;

    // Write .ready marker for the steam-launcher service
    guest_agent::write_file(vm_name, &format!("{dir}/.ready"), b"1")?;

    Ok(())
}

/// Switch a VM to a different Steam account.
///
/// 1. Kills Steam process
/// 2. Injects new session files
/// 3. Triggers Steam restart via systemd
pub fn switch_account(
    vm_name: &str,
    session: &SteamSession,
    config_dir: Option<&str>,
) -> Result<(), SessionError> {
    // Kill existing Steam process
    let kill_result = guest_agent::exec(vm_name, "pkill -TERM -f steam || true");
    if let Err(e) = kill_result {
        eprintln!("Warning: could not kill Steam in VM '{vm_name}': {e}");
    }

    // Wait for Steam to exit
    std::thread::sleep(std::time::Duration::from_secs(3));

    // Inject new session
    inject_session(vm_name, session, config_dir)?;

    // Restart the Steam farming service
    guest_agent::exec(vm_name, "systemctl --user restart steam-farm.service || true")?;

    Ok(())
}

/// Check if Steam is currently running inside a VM.
pub fn is_steam_running(vm_name: &str) -> Result<bool, SessionError> {
    let result = guest_agent::exec(vm_name, "pgrep -c steam 2>/dev/null || echo 0")?;
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
    fn test_config_vdf_generation() {
        let session = sample_session();
        let vdf = generate_config_vdf(&session);
        assert!(vdf.contains("\"AutoLoginUser\"\t\t\"testuser123\""));
        assert!(vdf.contains("\"SteamID\"\t\t\"76561198012345678\""));
        assert!(vdf.contains("\"Token\"\t\t\"eyJhbGciOiJFZERTQSJ9.test_token_data\""));
    }

    #[test]
    fn test_loginusers_vdf_generation() {
        let session = sample_session();
        let vdf = generate_loginusers_vdf(&session);
        assert!(vdf.contains("\"AccountName\"\t\t\"testuser123\""));
        assert!(vdf.contains("\"PersonaName\"\t\t\"TestPlayer\""));
        assert!(vdf.contains("\"RememberPassword\"\t\t\"1\""));
        assert!(vdf.contains("\"AllowAutoLogin\"\t\t\"1\""));
        assert!(vdf.contains("\"76561198012345678\""));
    }

    #[test]
    fn test_session_serialization() {
        let session = sample_session();
        let json = serde_json::to_string(&session).unwrap();
        let parsed: SteamSession = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.account_name, "testuser123");
        assert_eq!(parsed.steam_id, "76561198012345678");
    }

    #[test]
    fn test_config_vdf_structure() {
        let session = sample_session();
        let vdf = generate_config_vdf(&session);
        // Verify VDF structure has proper nesting
        assert!(vdf.starts_with("\"InstallConfigStore\""));
        assert!(vdf.contains("\"Software\""));
        assert!(vdf.contains("\"Valve\""));
        assert!(vdf.contains("\"Steam\""));
        assert!(vdf.contains("\"ConnectCache\""));
    }

    #[test]
    fn test_loginusers_vdf_structure() {
        let session = sample_session();
        let vdf = generate_loginusers_vdf(&session);
        assert!(vdf.starts_with("\"users\""));
        assert!(vdf.contains("\"MostRecent\"\t\t\"1\""));
    }
}
