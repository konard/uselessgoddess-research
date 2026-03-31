use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use hmac::{Hmac, Mac};
use rsa::{BigUint, Pkcs1v15Encrypt, RsaPublicKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

type HmacSha1 = Hmac<sha1::Sha1>;

/// Steam Web API base URL.
const STEAM_API_BASE: &str = "https://api.steampowered.com";

/// Character set for Steam Guard TOTP codes.
const STEAM_GUARD_CHARS: &[u8] = b"23456789BCDFGHJKMNPQRTVWXY";

/// Steam Guard code type for device-based TOTP.
const GUARD_TYPE_DEVICE_CODE: u32 = 3;

#[derive(Debug, Error)]
pub enum SteamAuthError {
    #[error("RSA key fetch failed: {0}")]
    RsaKeyFetch(String),
    #[error("RSA encryption failed: {0}")]
    RsaEncrypt(String),
    #[error("login begin failed: {0}")]
    LoginBegin(String),
    #[error("guard code submit failed: {0}")]
    GuardSubmit(String),
    #[error("poll failed: {0}")]
    Poll(String),
    #[error("login timed out after {0} attempts")]
    Timeout(u32),
    #[error("Steam Guard required but no shared_secret provided")]
    GuardRequired,
}

/// RSA public key response from Steam API.
#[derive(Debug, Deserialize)]
struct RsaKeyResponse {
    response: RsaKeyData,
}

#[derive(Debug, Deserialize)]
struct RsaKeyData {
    publickey_mod: String,
    publickey_exp: String,
    timestamp: String,
}

/// Response from BeginAuthSessionViaCredentials.
#[derive(Debug, Deserialize)]
struct BeginAuthResponse {
    response: BeginAuthData,
}

#[derive(Debug, Deserialize)]
struct BeginAuthData {
    client_id: String,
    request_id: String,
    #[serde(default)]
    interval: f64,
    #[serde(default)]
    allowed_confirmations: Vec<AllowedConfirmation>,
    steamid: String,
}

#[derive(Debug, Deserialize)]
struct AllowedConfirmation {
    confirmation_type: u32,
}

/// Response from PollAuthSessionStatus.
#[derive(Debug, Deserialize)]
struct PollResponse {
    response: PollData,
}

#[derive(Debug, Deserialize)]
struct PollData {
    #[serde(default)]
    refresh_token: String,
    #[serde(default)]
    access_token: String,
    #[serde(default)]
    account_name: String,
}

/// Credentials for Steam login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamCredentials {
    pub username: String,
    pub password: String,
    /// Base64-encoded shared secret for TOTP generation.
    pub shared_secret: Option<String>,
}

/// Result of a successful Steam login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub refresh_token: String,
    pub access_token: String,
    pub steam_id: String,
    pub account_name: String,
}

/// Generate a Steam Guard TOTP code from a shared secret.
///
/// Algorithm matches `node-steam-totp`:
/// 1. Decode base64 shared_secret
/// 2. Compute time = unix_timestamp / 30
/// 3. HMAC-SHA1(secret, time_bytes)
/// 4. Dynamic truncation to 5-char code using Steam's character set
pub fn generate_steam_guard_code(shared_secret: &str, time_offset: i64) -> Result<String, SteamAuthError> {
    let secret = BASE64.decode(shared_secret).map_err(|e| {
        SteamAuthError::GuardSubmit(format!("invalid shared_secret base64: {e}"))
    })?;

    let time = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + time_offset) as u64;
    let time_step = time / 30;

    // 8-byte big-endian buffer with time_step in the lower 4 bytes
    let mut buffer = [0u8; 8];
    buffer[4..8].copy_from_slice(&(time_step as u32).to_be_bytes());

    let mut mac =
        HmacSha1::new_from_slice(&secret).map_err(|e| SteamAuthError::GuardSubmit(e.to_string()))?;
    mac.update(&buffer);
    let hmac_result = mac.finalize().into_bytes();

    // Dynamic truncation
    let start = (hmac_result[19] & 0x0F) as usize;
    let fullcode_bytes = &hmac_result[start..start + 4];
    let mut fullcode = u32::from_be_bytes([
        fullcode_bytes[0],
        fullcode_bytes[1],
        fullcode_bytes[2],
        fullcode_bytes[3],
    ]) & 0x7FFFFFFF;

    let charset_len = STEAM_GUARD_CHARS.len() as u32;
    let mut code = String::with_capacity(5);
    for _ in 0..5 {
        code.push(STEAM_GUARD_CHARS[(fullcode % charset_len) as usize] as char);
        fullcode /= charset_len;
    }

    Ok(code)
}

/// Perform a full Steam login with username/password and optional TOTP.
///
/// Returns a `LoginResult` containing the refresh token that can be used
/// for session injection into containers.
pub fn login(credentials: &SteamCredentials) -> Result<LoginResult, SteamAuthError> {
    // Step 1: Get RSA public key for password encryption
    let rsa_url = format!(
        "{STEAM_API_BASE}/IAuthenticationService/GetPasswordRSAPublicKey/v1/?account_name={}",
        &credentials.username
    );

    let rsa_resp: RsaKeyResponse = ureq::get(&rsa_url)
        .call()
        .map_err(|e| SteamAuthError::RsaKeyFetch(e.to_string()))?
        .body_mut()
        .read_json()
        .map_err(|e| SteamAuthError::RsaKeyFetch(e.to_string()))?;

    let rsa_data = &rsa_resp.response;

    // Step 2: Encrypt password with RSA public key
    let encrypted_password = encrypt_password_rsa(
        &credentials.password,
        &rsa_data.publickey_mod,
        &rsa_data.publickey_exp,
    )?;

    // Step 3: Begin auth session with credentials
    let begin_url = format!(
        "{STEAM_API_BASE}/IAuthenticationService/BeginAuthSessionViaCredentials/v1/"
    );

    let begin_form: Vec<(&str, &str)> = vec![
        ("account_name", &credentials.username),
        ("encrypted_password", &encrypted_password),
        ("encryption_timestamp", &rsa_data.timestamp),
        ("remember_login", "true"),
        ("persistence", "1"),
        ("website_id", "Community"),
    ];
    let begin_resp: BeginAuthResponse = ureq::post(&begin_url)
        .send_form(begin_form)
        .map_err(|e| SteamAuthError::LoginBegin(e.to_string()))?
        .body_mut()
        .read_json()
        .map_err(|e| SteamAuthError::LoginBegin(e.to_string()))?;

    let auth_data = &begin_resp.response;

    // Step 4: Check if Steam Guard is needed and submit TOTP code
    let needs_device_code = auth_data
        .allowed_confirmations
        .iter()
        .any(|c| c.confirmation_type == GUARD_TYPE_DEVICE_CODE);

    if needs_device_code {
        let shared_secret = credentials
            .shared_secret
            .as_deref()
            .ok_or(SteamAuthError::GuardRequired)?;

        let code = generate_steam_guard_code(shared_secret, 0)?;

        let guard_url = format!(
            "{STEAM_API_BASE}/IAuthenticationService/UpdateAuthSessionWithSteamGuardCode/v1/"
        );

        let code_type_str = GUARD_TYPE_DEVICE_CODE.to_string();
        let guard_form: Vec<(&str, &str)> = vec![
            ("client_id", &auth_data.client_id),
            ("steamid", &auth_data.steamid),
            ("code", &code),
            ("code_type", &code_type_str),
        ];
        ureq::post(&guard_url)
            .send_form(guard_form)
            .map_err(|e| SteamAuthError::GuardSubmit(e.to_string()))?;
    }

    // Step 5: Poll for auth session status (get tokens)
    let poll_url = format!(
        "{STEAM_API_BASE}/IAuthenticationService/PollAuthSessionStatus/v1/"
    );

    let poll_interval = if auth_data.interval > 0.0 {
        auth_data.interval
    } else {
        5.0
    };

    let max_attempts = 30;
    for attempt in 0..max_attempts {
        std::thread::sleep(std::time::Duration::from_secs_f64(poll_interval));

        let poll_form: Vec<(&str, &str)> = vec![
            ("client_id", &auth_data.client_id),
            ("request_id", &auth_data.request_id),
        ];
        let poll_resp: PollResponse = ureq::post(&poll_url)
            .send_form(poll_form)
            .map_err(|e| SteamAuthError::Poll(e.to_string()))?
            .body_mut()
            .read_json()
            .map_err(|e| SteamAuthError::Poll(format!("attempt {attempt}: {e}")))?;

        let poll_data = &poll_resp.response;

        if !poll_data.refresh_token.is_empty() {
            return Ok(LoginResult {
                refresh_token: poll_data.refresh_token.clone(),
                access_token: poll_data.access_token.clone(),
                steam_id: auth_data.steamid.clone(),
                account_name: if poll_data.account_name.is_empty() {
                    credentials.username.clone()
                } else {
                    poll_data.account_name.clone()
                },
            });
        }
    }

    Err(SteamAuthError::Timeout(max_attempts))
}

/// Encrypt a password using an RSA public key from Steam.
fn encrypt_password_rsa(
    password: &str,
    modulus_hex: &str,
    exponent_hex: &str,
) -> Result<String, SteamAuthError> {
    let modulus = BigUint::parse_bytes(modulus_hex.as_bytes(), 16)
        .ok_or_else(|| SteamAuthError::RsaEncrypt("invalid modulus hex".into()))?;

    let exponent = BigUint::parse_bytes(exponent_hex.as_bytes(), 16)
        .ok_or_else(|| SteamAuthError::RsaEncrypt("invalid exponent hex".into()))?;

    let public_key = RsaPublicKey::new(modulus, exponent)
        .map_err(|e| SteamAuthError::RsaEncrypt(e.to_string()))?;

    let mut rng = rsa::rand_core::OsRng;
    let encrypted = public_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, password.as_bytes())
        .map_err(|e| SteamAuthError::RsaEncrypt(e.to_string()))?;

    Ok(BASE64.encode(encrypted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steam_guard_code_format() {
        // Use a known test shared_secret (base64 encoded 20 random bytes)
        let test_secret = BASE64.encode(b"12345678901234567890");
        let code = generate_steam_guard_code(&test_secret, 0).unwrap();

        assert_eq!(code.len(), 5, "Steam guard code must be 5 characters");
        for ch in code.chars() {
            assert!(
                STEAM_GUARD_CHARS.contains(&(ch as u8)),
                "character '{ch}' not in Steam guard charset"
            );
        }
    }

    #[test]
    fn test_steam_guard_code_deterministic() {
        let test_secret = BASE64.encode(b"12345678901234567890");
        // Same secret and offset should produce the same code
        let code1 = generate_steam_guard_code(&test_secret, 0).unwrap();
        let code2 = generate_steam_guard_code(&test_secret, 0).unwrap();
        assert_eq!(code1, code2);
    }

    #[test]
    fn test_steam_guard_code_different_secrets() {
        let secret1 = BASE64.encode(b"12345678901234567890");
        let secret2 = BASE64.encode(b"abcdefghijklmnopqrst");
        let code1 = generate_steam_guard_code(&secret1, 0).unwrap();
        let code2 = generate_steam_guard_code(&secret2, 0).unwrap();
        // Extremely unlikely to be equal with different secrets
        // (not guaranteed, but practically always different)
        assert_ne!(code1, code2);
    }

    #[test]
    fn test_steam_guard_invalid_secret() {
        let result = generate_steam_guard_code("not-valid-base64!!!", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_steam_guard_charset() {
        assert_eq!(STEAM_GUARD_CHARS.len(), 26);
        // Verify no ambiguous characters (0, 1, I, L, O, etc.)
        assert!(!STEAM_GUARD_CHARS.contains(&b'0'));
        assert!(!STEAM_GUARD_CHARS.contains(&b'1'));
        assert!(!STEAM_GUARD_CHARS.contains(&b'I'));
        assert!(!STEAM_GUARD_CHARS.contains(&b'L'));
        assert!(!STEAM_GUARD_CHARS.contains(&b'O'));
    }

    #[test]
    fn test_credentials_serialization() {
        let creds = SteamCredentials {
            username: "testuser".into(),
            password: "testpass".into(),
            shared_secret: Some("dGVzdHNlY3JldA==".into()),
        };
        let json = serde_json::to_string(&creds).unwrap();
        let parsed: SteamCredentials = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.username, "testuser");
        assert_eq!(parsed.shared_secret.as_deref(), Some("dGVzdHNlY3JldA=="));
    }

    #[test]
    fn test_login_result_serialization() {
        let result = LoginResult {
            refresh_token: "eyJhbGciOiJFZERTQSJ9.test".into(),
            access_token: "access_test".into(),
            steam_id: "76561198012345678".into(),
            account_name: "testuser".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("refresh_token"));
        assert!(json.contains("76561198012345678"));
    }
}
