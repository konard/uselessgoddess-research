use crate::container::exec as container_exec;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("container exec error: {0}")]
    Exec(#[from] container_exec::ContainerExecError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamSession {
    pub account_name: String,
    pub refresh_token: String,
    pub steam_id: String,
    pub persona_name: String,
}

const STEAM_CONFIG_DIR: &str = "/home/farmuser/.steam/steam/config";

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

pub fn inject_session(
    container_name: &str,
    session: &SteamSession,
    config_dir: Option<&str>,
) -> Result<(), SessionError> {
    let dir = config_dir.unwrap_or(STEAM_CONFIG_DIR);

    container_exec::exec(container_name, &format!("mkdir -p '{dir}'"))?;

    let config_vdf = generate_config_vdf(session);
    container_exec::write_file(
        container_name,
        &format!("{dir}/config.vdf"),
        config_vdf.as_bytes(),
    )?;

    let loginusers_vdf = generate_loginusers_vdf(session);
    container_exec::write_file(
        container_name,
        &format!("{dir}/loginusers.vdf"),
        loginusers_vdf.as_bytes(),
    )?;

    container_exec::exec(
        container_name,
        &format!("chown -R farmuser:farmuser '{dir}'"),
    )?;

    container_exec::write_file(container_name, &format!("{dir}/.ready"), b"1")?;

    Ok(())
}

pub fn switch_account(
    container_name: &str,
    session: &SteamSession,
    config_dir: Option<&str>,
) -> Result<(), SessionError> {
    let kill_result = container_exec::exec(container_name, "pkill -TERM -f steam || true");
    if let Err(e) = kill_result {
        eprintln!("Warning: could not kill Steam in container '{container_name}': {e}");
    }

    std::thread::sleep(std::time::Duration::from_secs(3));

    inject_session(container_name, session, config_dir)?;

    Ok(())
}

pub fn is_steam_running(container_name: &str) -> Result<bool, SessionError> {
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
