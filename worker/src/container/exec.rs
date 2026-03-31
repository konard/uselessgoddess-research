use std::process::Command;
use thiserror::Error;

use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum ContainerExecError {
    #[error("container `{0}` is not running")]
    NotRunning(String),
    #[error("exec failed on container `{container}`: {reason}")]
    ExecFailed { container: String, reason: String },
    #[error("file operation failed on container `{container}`: {reason}")]
    FileError { container: String, reason: String },
    #[error("docker command failed: {0}")]
    Docker(String),
}

/// Result of a command executed inside the container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Check if a container is running.
pub fn is_running(container_name: &str) -> Result<bool, ContainerExecError> {
    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.State.Running}}", container_name])
        .output()
        .map_err(|e| ContainerExecError::Docker(e.to_string()))?;

    if !output.status.success() {
        return Ok(false);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.trim() == "true")
}

/// Execute a command inside a running container.
///
/// Uses `docker exec` with `/bin/sh -c` for shell expansion support.
/// This is the container equivalent of `guest_agent::exec`.
pub fn exec(container_name: &str, cmd: &str) -> Result<ExecResult, ContainerExecError> {
    let output = Command::new("docker")
        .args(["exec", container_name, "/bin/sh", "-c", cmd])
        .output()
        .map_err(|e| ContainerExecError::Docker(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Detect "not running" errors
    if !output.status.success() && stderr.contains("is not running") {
        return Err(ContainerExecError::NotRunning(container_name.to_string()));
    }

    let exit_code = output.status.code().unwrap_or(-1);

    Ok(ExecResult {
        exit_code,
        stdout,
        stderr,
    })
}

/// Execute a command as a specific user inside the container.
pub fn exec_as_user(
    container_name: &str,
    user: &str,
    cmd: &str,
) -> Result<ExecResult, ContainerExecError> {
    let output = Command::new("docker")
        .args(["exec", "-u", user, container_name, "/bin/sh", "-c", cmd])
        .output()
        .map_err(|e| ContainerExecError::Docker(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() && stderr.contains("is not running") {
        return Err(ContainerExecError::NotRunning(container_name.to_string()));
    }

    Ok(ExecResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout,
        stderr,
    })
}

/// Write a file inside the container using `docker exec` + `tee`.
pub fn write_file(
    container_name: &str,
    guest_path: &str,
    data: &[u8],
) -> Result<(), ContainerExecError> {
    // Ensure parent directory exists
    let parent = std::path::Path::new(guest_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if !parent.is_empty() {
        let _ = exec(container_name, &format!("mkdir -p '{parent}'"));
    }

    // Write via docker exec stdin -> tee
    let mut child = Command::new("docker")
        .args(["exec", "-i", container_name, "tee", guest_path])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ContainerExecError::Docker(e.to_string()))?;

    if let Some(ref mut stdin) = child.stdin {
        use std::io::Write;
        stdin.write_all(data).map_err(|e| {
            ContainerExecError::FileError {
                container: container_name.to_string(),
                reason: format!("write to stdin: {e}"),
            }
        })?;
    }
    // Drop stdin to close pipe
    drop(child.stdin.take());

    let output = child.wait_with_output().map_err(|e| {
        ContainerExecError::Docker(e.to_string())
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerExecError::FileError {
            container: container_name.to_string(),
            reason: format!("tee failed: {stderr}"),
        });
    }

    Ok(())
}

/// Read a file from inside the container using `docker exec cat`.
pub fn read_file(container_name: &str, guest_path: &str) -> Result<Vec<u8>, ContainerExecError> {
    let output = Command::new("docker")
        .args(["exec", container_name, "cat", guest_path])
        .output()
        .map_err(|e| ContainerExecError::Docker(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerExecError::FileError {
            container: container_name.to_string(),
            reason: format!("cat {guest_path}: {stderr}"),
        });
    }

    Ok(output.stdout)
}

/// Copy a file from host into the container.
///
/// Uses `docker cp` which is more efficient than exec+tee for large files.
pub fn copy_to_container(
    container_name: &str,
    host_path: &str,
    container_path: &str,
) -> Result<(), ContainerExecError> {
    let output = Command::new("docker")
        .args(["cp", host_path, &format!("{container_name}:{container_path}")])
        .output()
        .map_err(|e| ContainerExecError::Docker(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerExecError::FileError {
            container: container_name.to_string(),
            reason: format!("docker cp: {stderr}"),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exec_result_serialization() {
        let result = ExecResult {
            exit_code: 0,
            stdout: "hello\n".into(),
            stderr: "".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"exit_code\":0"));
        let parsed: ExecResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.stdout, "hello\n");
    }

    #[test]
    fn test_is_running_nonexistent() {
        // This should return false (or error) for a non-existent container
        // Only works if docker is installed; skip gracefully otherwise
        match is_running("vmctl-nonexistent-test-container") {
            Ok(running) => assert!(!running),
            Err(ContainerExecError::Docker(_)) => {
                // Docker not installed, skip
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
}
