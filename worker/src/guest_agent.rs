//! QEMU Guest Agent interface via `virsh qemu-agent-command`.
//!
//! Provides typed wrappers around the JSON-based guest-agent protocol
//! for executing commands, transferring files, and querying VM state
//! without requiring network connectivity inside the guest.

use serde::{Deserialize, Serialize};
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum GuestAgentError {
    #[error("guest agent not responding for VM `{0}` (is qemu-guest-agent installed and running?)")]
    NotResponding(String),
    #[error("guest-exec failed on VM `{vm}`: {reason}")]
    ExecFailed { vm: String, reason: String },
    #[error("file operation failed on VM `{vm}`: {reason}")]
    FileError { vm: String, reason: String },
    #[error("virsh command failed: {0}")]
    Virsh(String),
    #[error("invalid JSON response: {0}")]
    Json(String),
}

/// Result of a command executed inside the guest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Network interface information from the guest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuestInterface {
    pub name: String,
    pub hardware_address: String,
    pub ip_addresses: Vec<String>,
}

/// Send a raw guest-agent command via virsh and return the JSON response.
fn ga_command(vm_name: &str, json_cmd: &str) -> Result<String, GuestAgentError> {
    let output = Command::new("virsh")
        .args(["qemu-agent-command", vm_name, json_cmd])
        .output()
        .map_err(|e| GuestAgentError::Virsh(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if stderr.contains("agent is not connected")
            || stderr.contains("Guest agent is not responding")
            || stderr.contains("QEMU guest agent is not connected")
        {
            return Err(GuestAgentError::NotResponding(vm_name.to_string()));
        }
        return Err(GuestAgentError::Virsh(stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Ping the guest agent to check availability.
pub fn ping(vm_name: &str) -> Result<bool, GuestAgentError> {
    let result = ga_command(vm_name, r#"{"execute":"guest-ping"}"#);
    match result {
        Ok(_) => Ok(true),
        Err(GuestAgentError::NotResponding(_)) => Ok(false),
        Err(e) => Err(e),
    }
}

/// Execute a command inside the guest and wait for the result.
///
/// Uses `guest-exec` to start the process and `guest-exec-status` to poll
/// for completion. The command is run as `["/bin/sh", "-c", cmd]` for
/// shell expansion support.
pub fn exec(vm_name: &str, cmd: &str) -> Result<ExecResult, GuestAgentError> {
    // Build guest-exec command with base64-encoded arguments
    let exec_cmd = serde_json::json!({
        "execute": "guest-exec",
        "arguments": {
            "path": "/bin/sh",
            "arg": ["-c", cmd],
            "capture-output": true
        }
    });

    let resp = ga_command(vm_name, &exec_cmd.to_string())?;
    let resp_json: serde_json::Value =
        serde_json::from_str(&resp).map_err(|e| GuestAgentError::Json(e.to_string()))?;

    let pid = resp_json["return"]["pid"]
        .as_i64()
        .ok_or_else(|| GuestAgentError::Json("missing pid in guest-exec response".into()))?;

    // Poll for completion (guest-exec-status)
    let max_attempts = 60;
    for _ in 0..max_attempts {
        let status_cmd = serde_json::json!({
            "execute": "guest-exec-status",
            "arguments": { "pid": pid }
        });

        let status_resp = ga_command(vm_name, &status_cmd.to_string())?;
        let status_json: serde_json::Value = serde_json::from_str(&status_resp)
            .map_err(|e| GuestAgentError::Json(e.to_string()))?;

        let ret = &status_json["return"];
        if ret["exited"].as_bool() == Some(true) {
            let exit_code = ret["exitcode"].as_i64().unwrap_or(-1) as i32;
            let stdout = ret["out-data"]
                .as_str()
                .map(decode_base64)
                .unwrap_or_default();
            let stderr = ret["err-data"]
                .as_str()
                .map(decode_base64)
                .unwrap_or_default();

            return Ok(ExecResult {
                exit_code,
                stdout,
                stderr,
            });
        }

        std::thread::sleep(std::time::Duration::from_secs(1));
    }

    Err(GuestAgentError::ExecFailed {
        vm: vm_name.to_string(),
        reason: format!("command did not complete within {max_attempts}s: {cmd}"),
    })
}

/// Write a file inside the guest VM using guest-file-open/write/close.
pub fn write_file(
    vm_name: &str,
    guest_path: &str,
    data: &[u8],
) -> Result<(), GuestAgentError> {
    // Open file for writing
    let open_cmd = serde_json::json!({
        "execute": "guest-file-open",
        "arguments": {
            "path": guest_path,
            "mode": "w"
        }
    });
    let resp = ga_command(vm_name, &open_cmd.to_string())?;
    let resp_json: serde_json::Value =
        serde_json::from_str(&resp).map_err(|e| GuestAgentError::Json(e.to_string()))?;

    let handle = resp_json["return"]
        .as_i64()
        .ok_or_else(|| GuestAgentError::Json("missing file handle".into()))?;

    // Write data (base64-encoded)
    let encoded = encode_base64(data);
    let write_cmd = serde_json::json!({
        "execute": "guest-file-write",
        "arguments": {
            "handle": handle,
            "buf-b64": encoded
        }
    });
    ga_command(vm_name, &write_cmd.to_string())?;

    // Close file
    let close_cmd = serde_json::json!({
        "execute": "guest-file-close",
        "arguments": { "handle": handle }
    });
    ga_command(vm_name, &close_cmd.to_string())?;

    Ok(())
}

/// Read a file from inside the guest VM.
pub fn read_file(vm_name: &str, guest_path: &str) -> Result<Vec<u8>, GuestAgentError> {
    let open_cmd = serde_json::json!({
        "execute": "guest-file-open",
        "arguments": {
            "path": guest_path,
            "mode": "r"
        }
    });
    let resp = ga_command(vm_name, &open_cmd.to_string())?;
    let resp_json: serde_json::Value =
        serde_json::from_str(&resp).map_err(|e| GuestAgentError::Json(e.to_string()))?;

    let handle = resp_json["return"]
        .as_i64()
        .ok_or_else(|| GuestAgentError::Json("missing file handle".into()))?;

    // Read in chunks
    let mut result = Vec::new();
    loop {
        let read_cmd = serde_json::json!({
            "execute": "guest-file-read",
            "arguments": {
                "handle": handle,
                "count": 65536
            }
        });
        let read_resp = ga_command(vm_name, &read_cmd.to_string())?;
        let read_json: serde_json::Value = serde_json::from_str(&read_resp)
            .map_err(|e| GuestAgentError::Json(e.to_string()))?;

        let ret = &read_json["return"];
        let buf = ret["buf-b64"]
            .as_str()
            .map(decode_base64_bytes)
            .unwrap_or_default();
        let eof = ret["eof"].as_bool().unwrap_or(true);

        result.extend_from_slice(&buf);
        if eof {
            break;
        }
    }

    // Close
    let close_cmd = serde_json::json!({
        "execute": "guest-file-close",
        "arguments": { "handle": handle }
    });
    ga_command(vm_name, &close_cmd.to_string())?;

    Ok(result)
}

/// Query network interfaces inside the guest.
pub fn get_network_interfaces(vm_name: &str) -> Result<Vec<GuestInterface>, GuestAgentError> {
    let resp = ga_command(
        vm_name,
        r#"{"execute":"guest-network-get-interfaces"}"#,
    )?;
    let resp_json: serde_json::Value =
        serde_json::from_str(&resp).map_err(|e| GuestAgentError::Json(e.to_string()))?;

    let ifaces = resp_json["return"]
        .as_array()
        .ok_or_else(|| GuestAgentError::Json("expected array of interfaces".into()))?;

    let mut result = Vec::new();
    for iface in ifaces {
        let name = iface["name"].as_str().unwrap_or("unknown").to_string();
        let hw_addr = iface["hardware-address"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let mut ips = Vec::new();
        if let Some(addrs) = iface["ip-addresses"].as_array() {
            for addr in addrs {
                if let Some(ip) = addr["ip-address"].as_str() {
                    ips.push(ip.to_string());
                }
            }
        }

        result.push(GuestInterface {
            name,
            hardware_address: hw_addr,
            ip_addresses: ips,
        });
    }
    Ok(result)
}

/// Simple base64 encoding (no external dependency).
fn encode_base64(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Decode base64 string to UTF-8 (lossy).
fn decode_base64(s: &str) -> String {
    String::from_utf8_lossy(&decode_base64_bytes(s)).to_string()
}

/// Decode base64 string to bytes.
fn decode_base64_bytes(s: &str) -> Vec<u8> {
    fn val(c: u8) -> u8 {
        match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => 0,
        }
    }

    let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'\n' && b != b'\r').collect();
    let mut result = Vec::with_capacity(bytes.len() * 3 / 4);

    for chunk in bytes.chunks(4) {
        if chunk.len() < 2 {
            break;
        }
        let b0 = val(chunk[0]);
        let b1 = val(chunk[1]);
        let b2 = if chunk.len() > 2 && chunk[2] != b'=' {
            val(chunk[2])
        } else {
            0
        };
        let b3 = if chunk.len() > 3 && chunk[3] != b'=' {
            val(chunk[3])
        } else {
            0
        };

        result.push((b0 << 2) | (b1 >> 4));
        if chunk.len() > 2 && chunk[2] != b'=' {
            result.push((b1 << 4) | (b2 >> 2));
        }
        if chunk.len() > 3 && chunk[3] != b'=' {
            result.push((b2 << 6) | b3);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, Guest Agent!";
        let encoded = encode_base64(data);
        let decoded = decode_base64_bytes(&encoded);
        assert_eq!(decoded, data);
    }

    #[test]
    fn test_base64_empty() {
        assert_eq!(encode_base64(b""), "");
        assert_eq!(decode_base64_bytes(""), Vec::<u8>::new());
    }

    #[test]
    fn test_base64_padding() {
        // 1 byte -> 4 chars with 2 padding
        let encoded = encode_base64(b"A");
        assert_eq!(encoded, "QQ==");
        assert_eq!(decode_base64_bytes("QQ=="), b"A");

        // 2 bytes -> 4 chars with 1 padding
        let encoded = encode_base64(b"AB");
        assert_eq!(encoded, "QUI=");
        assert_eq!(decode_base64_bytes("QUI="), b"AB");

        // 3 bytes -> 4 chars, no padding
        let encoded = encode_base64(b"ABC");
        assert_eq!(encoded, "QUJD");
        assert_eq!(decode_base64_bytes("QUJD"), b"ABC");
    }

    #[test]
    fn test_exec_result_serialization() {
        let result = ExecResult {
            exit_code: 0,
            stdout: "output".into(),
            stderr: "".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"exit_code\":0"));
        let parsed: ExecResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.exit_code, 0);
        assert_eq!(parsed.stdout, "output");
    }

    #[test]
    fn test_guest_interface_serialization() {
        let iface = GuestInterface {
            name: "eth0".into(),
            hardware_address: "00:1b:21:aa:bb:cc".into(),
            ip_addresses: vec!["192.168.122.100".into()],
        };
        let json = serde_json::to_string(&iface).unwrap();
        let parsed: GuestInterface = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "eth0");
        assert_eq!(parsed.ip_addresses.len(), 1);
    }

    #[test]
    fn test_base64_known_vectors() {
        // Standard base64 test vectors
        assert_eq!(encode_base64(b"Man"), "TWFu");
        assert_eq!(decode_base64("TWFu"), "Man");
        assert_eq!(encode_base64(b"Ma"), "TWE=");
        assert_eq!(decode_base64("TWE="), "Ma");
    }
}
