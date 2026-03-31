//! Hardware spoofing verification for containers.
//!
//! Container equivalent of `verify.rs`. Checks that spoofed hardware
//! identifiers are visible inside the running container.

use crate::container_exec;
use crate::container_spoof;
use crate::spoof::HwIdentity;

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("container exec error: {0}")]
    Exec(#[from] container_exec::ContainerExecError),
}

/// Result of a single spoofing check.
#[derive(Debug, Clone, Serialize)]
pub struct SpoofCheck {
    pub component: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

/// Complete verification report.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub container_name: String,
    pub checks: Vec<SpoofCheck>,
    pub all_passed: bool,
}

/// Verify hardware spoofing inside a running container.
///
/// Runs checks via `docker exec` and compares actual values against expected.
pub fn verify_spoofing(
    container_name: &str,
    expected: &HwIdentity,
) -> Result<VerifyReport, VerifyError> {
    let verification_checks = container_spoof::verify_args(container_name, expected);
    let expected_machine_id = container_spoof::generate_machine_id(container_name);

    let mut checks = Vec::new();

    for (component, expected_value, cmd) in verification_checks {
        let result = container_exec::exec(container_name, &cmd)?;
        let actual = result.stdout.trim().to_string();

        let passed = match component {
            "machine-id" => actual.trim() == expected_machine_id.trim(),
            "mac_address" => actual.to_lowercase() == expected_value.to_lowercase(),
            _ => actual.trim() == expected_value.trim(),
        };

        checks.push(SpoofCheck {
            component: component.to_string(),
            expected: expected_value,
            actual,
            passed,
        });
    }

    // Additional check: GPU is accessible
    let gpu_result = container_exec::exec(
        container_name,
        "ls /dev/dri/renderD128 2>/dev/null && echo OK || echo MISSING",
    )?;
    let gpu_accessible = gpu_result.stdout.trim().contains("OK");
    checks.push(SpoofCheck {
        component: "gpu_render_node".to_string(),
        expected: "/dev/dri/renderD128 accessible".to_string(),
        actual: if gpu_accessible {
            "accessible".to_string()
        } else {
            "not found".to_string()
        },
        passed: gpu_accessible,
    });

    // Check Vulkan availability
    let vulkan_result = container_exec::exec(
        container_name,
        "vulkaninfo --summary 2>/dev/null | head -5 || echo 'vulkaninfo not available'",
    )?;
    let vulkan_ok = !vulkan_result.stdout.contains("not available")
        && !vulkan_result.stdout.contains("ERROR");
    checks.push(SpoofCheck {
        component: "vulkan".to_string(),
        expected: "Vulkan available".to_string(),
        actual: vulkan_result.stdout.lines().next().unwrap_or("unknown").trim().to_string(),
        passed: vulkan_ok,
    });

    let all_passed = checks.iter().all(|c| c.passed);

    Ok(VerifyReport {
        container_name: container_name.to_string(),
        checks,
        all_passed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spoof_check_serialization() {
        let check = SpoofCheck {
            component: "machine-id".into(),
            expected: "abc123".into(),
            actual: "abc123".into(),
            passed: true,
        };
        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("\"passed\":true"));
        assert!(json.contains("\"component\":\"machine-id\""));
    }

    #[test]
    fn test_verify_report_serialization() {
        let report = VerifyReport {
            container_name: "test-container".into(),
            checks: vec![SpoofCheck {
                component: "mac_address".into(),
                expected: "a4:bb:6d:12:34:56".into(),
                actual: "a4:bb:6d:12:34:56".into(),
                passed: true,
            }],
            all_passed: true,
        };
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"all_passed\": true"));
        assert!(json.contains("\"container_name\": \"test-container\""));
    }
}
