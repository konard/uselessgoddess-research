//! Hardware spoofing verification inside VMs.
//!
//! Uses the QEMU Guest Agent to run diagnostic commands inside the VM
//! and compare the results against the expected spoofed identity.

use crate::guest_agent;
use crate::spoof::HwIdentity;

use serde::Serialize;

/// Result of a single spoofing check.
#[derive(Debug, Clone, Serialize)]
pub struct SpoofCheck {
    pub component: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

/// Full verification report for a VM.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyReport {
    pub vm_name: String,
    pub checks: Vec<SpoofCheck>,
    pub all_passed: bool,
}

/// Commands to run inside the guest for hardware verification.
struct VerifyCmd {
    component: &'static str,
    command: &'static str,
    extract: fn(&str) -> String,
}

/// Extract the first non-empty trimmed line from command output.
fn first_line(output: &str) -> String {
    output
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .to_string()
}

/// Extract MAC address from `ip link show` output.
///
/// Looks for lines containing `link/ether` and extracts the MAC.
fn extract_mac(output: &str) -> String {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("link/ether") {
            return trimmed
                .split_whitespace()
                .nth(1)
                .unwrap_or("")
                .to_string();
        }
    }
    String::new()
}

/// Extract disk serial from `lsblk` output.
///
/// Parses `lsblk -o NAME,SERIAL -n` format.
fn extract_disk_serial(output: &str) -> String {
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            // Return the serial (second column) for the first disk found
            return parts[1].to_string();
        }
    }
    String::new()
}

/// Extract a dmidecode field value from its output.
///
/// Looks for lines like `\tManufacturer: Dell Inc.` and returns the value.
fn extract_dmidecode_field(output: &str, field: &str) -> String {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(field) {
            let rest = rest.trim_start_matches(':').trim();
            return rest.to_string();
        }
    }
    String::new()
}

/// Verify hardware spoofing for a running VM by comparing guest-visible
/// hardware identifiers against the expected spoofed identity.
///
/// Requires:
/// - VM is running
/// - `qemu-guest-agent` is installed and active inside the guest
/// - Guest has `ip`, `lsblk`, and `dmidecode` commands available
pub fn verify_spoofing(
    vm_name: &str,
    expected: &HwIdentity,
) -> Result<VerifyReport, guest_agent::GuestAgentError> {
    // First check that the guest agent is responding
    let alive = guest_agent::ping(vm_name)?;
    if !alive {
        return Err(guest_agent::GuestAgentError::NotResponding(
            vm_name.to_string(),
        ));
    }

    let mut checks = Vec::new();

    // 1. MAC address check
    let mac_result = guest_agent::exec(vm_name, "ip link show 2>/dev/null")?;
    let actual_mac = extract_mac(&mac_result.stdout);
    checks.push(SpoofCheck {
        component: "MAC Address".into(),
        expected: expected.mac_address.clone(),
        actual: actual_mac.clone(),
        passed: actual_mac.eq_ignore_ascii_case(&expected.mac_address),
    });

    // 2. SMBIOS manufacturer
    let dmi_system =
        guest_agent::exec(vm_name, "dmidecode -t system 2>/dev/null || echo ''")?;
    let actual_mfr = extract_dmidecode_field(&dmi_system.stdout, "Manufacturer");
    checks.push(SpoofCheck {
        component: "SMBIOS Manufacturer".into(),
        expected: expected.smbios_manufacturer.clone(),
        actual: actual_mfr.clone(),
        passed: actual_mfr == expected.smbios_manufacturer,
    });

    // 3. SMBIOS product name
    let actual_product = extract_dmidecode_field(&dmi_system.stdout, "Product Name");
    checks.push(SpoofCheck {
        component: "SMBIOS Product".into(),
        expected: expected.smbios_product.clone(),
        actual: actual_product.clone(),
        passed: actual_product == expected.smbios_product,
    });

    // 4. SMBIOS serial
    let actual_serial = extract_dmidecode_field(&dmi_system.stdout, "Serial Number");
    checks.push(SpoofCheck {
        component: "SMBIOS Serial".into(),
        expected: expected.smbios_serial.clone(),
        actual: actual_serial.clone(),
        passed: actual_serial == expected.smbios_serial,
    });

    // 5. Disk serial
    let disk_result =
        guest_agent::exec(vm_name, "lsblk -o NAME,SERIAL -n 2>/dev/null || echo ''")?;
    let actual_disk_serial = extract_disk_serial(&disk_result.stdout);
    checks.push(SpoofCheck {
        component: "Disk Serial".into(),
        expected: expected.disk_serial.clone(),
        actual: actual_disk_serial.clone(),
        passed: actual_disk_serial == expected.disk_serial,
    });

    // 6. Hypervisor visibility check (should NOT detect hypervisor)
    let cpuid_result = guest_agent::exec(
        vm_name,
        "grep -c hypervisor /proc/cpuinfo 2>/dev/null || echo 0",
    )?;
    let hypervisor_visible = first_line(&cpuid_result.stdout) != "0";
    checks.push(SpoofCheck {
        component: "Hypervisor Hidden".into(),
        expected: "hidden (0)".into(),
        actual: if hypervisor_visible {
            "visible".into()
        } else {
            "hidden (0)".into()
        },
        passed: !hypervisor_visible,
    });

    // 7. NIC model check (should be e1000e, not virtio-net)
    let nic_result = guest_agent::exec(
        vm_name,
        "ls /sys/class/net/*/device/driver 2>/dev/null | head -5 || echo ''",
    )?;
    let nic_output = nic_result.stdout.trim().to_string();
    let uses_e1000e = nic_output.contains("e1000e");
    checks.push(SpoofCheck {
        component: "NIC Driver (e1000e)".into(),
        expected: "e1000e".into(),
        actual: if uses_e1000e {
            "e1000e".into()
        } else {
            nic_output
        },
        passed: uses_e1000e,
    });

    let all_passed = checks.iter().all(|c| c.passed);

    Ok(VerifyReport {
        vm_name: vm_name.to_string(),
        checks,
        all_passed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_mac() {
        let output = r#"2: enp1s0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc fq_codel state UP mode DEFAULT group default qlen 1000
    link/ether 3c:97:0e:ab:cd:ef brd ff:ff:ff:ff:ff:ff"#;
        assert_eq!(extract_mac(output), "3c:97:0e:ab:cd:ef");
    }

    #[test]
    fn test_extract_mac_empty() {
        assert_eq!(extract_mac("no ether here"), "");
    }

    #[test]
    fn test_extract_disk_serial() {
        let output = "vda   WDC12345678\nsda   \n";
        assert_eq!(extract_disk_serial(output), "WDC12345678");
    }

    #[test]
    fn test_extract_disk_serial_empty() {
        assert_eq!(extract_disk_serial("vda\n"), "");
    }

    #[test]
    fn test_extract_dmidecode_field() {
        let output = r#"System Information
	Manufacturer: Dell Inc.
	Product Name: OptiPlex 7080
	Version: Not Specified
	Serial Number: SVC1234567"#;
        assert_eq!(
            extract_dmidecode_field(output, "Manufacturer"),
            "Dell Inc."
        );
        assert_eq!(
            extract_dmidecode_field(output, "Product Name"),
            "OptiPlex 7080"
        );
        assert_eq!(
            extract_dmidecode_field(output, "Serial Number"),
            "SVC1234567"
        );
    }

    #[test]
    fn test_extract_dmidecode_missing_field() {
        assert_eq!(extract_dmidecode_field("nothing here", "Manufacturer"), "");
    }

    #[test]
    fn test_first_line() {
        assert_eq!(first_line("hello\nworld"), "hello");
        assert_eq!(first_line("\n\nfoo"), "foo");
        assert_eq!(first_line(""), "");
    }

    #[test]
    fn test_spoof_check_serialization() {
        let check = SpoofCheck {
            component: "MAC".into(),
            expected: "00:11:22:33:44:55".into(),
            actual: "00:11:22:33:44:55".into(),
            passed: true,
        };
        let json = serde_json::to_string(&check).unwrap();
        assert!(json.contains("\"passed\":true"));
    }

    #[test]
    fn test_verify_report_serialization() {
        let report = VerifyReport {
            vm_name: "test-vm".into(),
            checks: vec![SpoofCheck {
                component: "MAC".into(),
                expected: "aa:bb:cc:dd:ee:ff".into(),
                actual: "aa:bb:cc:dd:ee:ff".into(),
                passed: true,
            }],
            all_passed: true,
        };
        let json = serde_json::to_string_pretty(&report).unwrap();
        assert!(json.contains("\"all_passed\": true"));
        assert!(json.contains("test-vm"));
    }
}
