//! Host dependency checks for container mode.
//!
//! Container mode requires Docker and GPU render nodes instead of
//! QEMU, libvirt, and KVM.

use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DepError {
    #[error("{name}: not found — {detail}")]
    NotFound { name: String, detail: String },
    #[error("{name}: version {found} < required {required}")]
    VersionTooLow {
        name: String,
        found: String,
        required: String,
    },
    #[error("{name}: check failed — {detail}")]
    CheckFailed { name: String, detail: String },
}

/// Result of a single dependency check.
pub struct DepCheck {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

/// Check that Docker is installed and the daemon is running.
fn check_docker() -> Result<DepCheck, DepError> {
    let output = Command::new("docker").args(["version", "--format", "{{.Server.Version}}"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout).trim().to_string();
            Ok(DepCheck {
                name: "docker".into(),
                ok: true,
                detail: format!("Docker {version}"),
            })
        }
        Ok(o) => {
            let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
            if stderr.contains("permission denied") || stderr.contains("connect") {
                Err(DepError::CheckFailed {
                    name: "docker".into(),
                    detail: "Docker daemon not accessible. Is the user in the 'docker' group?".into(),
                })
            } else {
                Err(DepError::CheckFailed {
                    name: "docker".into(),
                    detail: stderr,
                })
            }
        }
        Err(_) => Err(DepError::NotFound {
            name: "docker".into(),
            detail: "Install Docker: https://docs.docker.com/engine/install/".into(),
        }),
    }
}

/// Check that GPU render nodes are available.
fn check_gpu() -> Result<DepCheck, DepError> {
    let render_path = std::path::Path::new("/dev/dri/renderD128");
    if render_path.exists() {
        // Try to identify the GPU
        let gpu_info = Command::new("sh")
            .args(["-c", "ls -la /dev/dri/renderD128 && cat /sys/class/drm/renderD128/device/vendor 2>/dev/null || echo unknown"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "present".to_string());

        Ok(DepCheck {
            name: "gpu".into(),
            ok: true,
            detail: format!("Render node: /dev/dri/renderD128 ({gpu_info})"),
        })
    } else {
        Err(DepError::NotFound {
            name: "gpu".into(),
            detail: "/dev/dri/renderD128 not found. Is the GPU driver loaded?".into(),
        })
    }
}

/// Check that the CS2 farm Docker image exists.
fn check_image(image: &str) -> Result<DepCheck, DepError> {
    let output = Command::new("docker")
        .args(["image", "inspect", image])
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(DepCheck {
            name: "docker-image".into(),
            ok: true,
            detail: format!("Image '{image}' found"),
        }),
        _ => Err(DepError::NotFound {
            name: "docker-image".into(),
            detail: format!("Image '{image}' not found. Build with: docker build -t {image} -f container/Dockerfile container/"),
        }),
    }
}

/// Check that /opt/cs2-shared exists (optional).
fn check_cs2_shared(path: &str) -> Result<DepCheck, DepError> {
    if std::path::Path::new(path).is_dir() {
        Ok(DepCheck {
            name: "cs2-shared".into(),
            ok: true,
            detail: format!("CS2 shared directory: {path}"),
        })
    } else {
        // Not an error — CS2 might not be installed yet
        Ok(DepCheck {
            name: "cs2-shared".into(),
            ok: false,
            detail: format!("{path} not found (CS2 not installed yet — optional for initial setup)"),
        })
    }
}

/// Run all container-mode dependency checks.
pub fn check_all(image: &str, cs2_dir: &str) -> (Vec<DepCheck>, Vec<DepError>) {
    let mut ok = Vec::new();
    let mut errors = Vec::new();

    for result in [check_docker(), check_gpu()] {
        match result {
            Ok(check) => ok.push(check),
            Err(e) => errors.push(e),
        }
    }

    match check_image(image) {
        Ok(check) => ok.push(check),
        Err(e) => errors.push(e),
    }

    match check_cs2_shared(cs2_dir) {
        Ok(check) => ok.push(check),
        Err(e) => errors.push(e),
    }

    (ok, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dep_error_display() {
        let err = DepError::NotFound {
            name: "docker".into(),
            detail: "not installed".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("docker"));
        assert!(msg.contains("not installed"));
    }

    #[test]
    fn test_dep_error_version() {
        let err = DepError::VersionTooLow {
            name: "docker".into(),
            found: "19.03".into(),
            required: "20.10".into(),
        };
        let msg = err.to_string();
        assert!(msg.contains("19.03"));
        assert!(msg.contains("20.10"));
    }
}
