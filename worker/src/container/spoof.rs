use crate::spoof::HwIdentity;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContainerSpoofError {
    #[error("I/O error creating spoof files: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpoofFiles {
    pub host_dir: PathBuf,
    pub machine_id: String,
    pub docker_args: Vec<String>,
}

const DMI_FILES: &[(&str, &str)] = &[
    ("sys_vendor", "smbios_manufacturer"),
    ("product_name", "smbios_product"),
    ("product_serial", "smbios_serial"),
    ("board_vendor", "smbios_manufacturer"),
    ("board_name", "smbios_product"),
    ("board_serial", "smbios_serial"),
    ("chassis_vendor", "smbios_manufacturer"),
    ("chassis_serial", "smbios_serial"),
];

pub fn generate_machine_id(container_name: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    container_name.hash(&mut hasher);
    let h1 = hasher.finish();

    h1.hash(&mut hasher);
    let h2 = hasher.finish();

    format!("{:016x}{:016x}", h1, h2)
}

pub fn create_spoof_files(
    spoof_dir: &str,
    container_name: &str,
    hw: &HwIdentity,
) -> Result<SpoofFiles, ContainerSpoofError> {
    let host_dir = PathBuf::from(spoof_dir).join(container_name);
    let dmi_dir = host_dir.join("dmi");

    std::fs::create_dir_all(&dmi_dir)?;

    for &(filename, field) in DMI_FILES {
        let value = match field {
            "smbios_manufacturer" => &hw.smbios_manufacturer,
            "smbios_product" => &hw.smbios_product,
            "smbios_serial" => &hw.smbios_serial,
            _ => continue,
        };
        let path = dmi_dir.join(filename);
        std::fs::write(&path, format!("{value}\n"))?;
    }

    let machine_id = generate_machine_id(container_name);
    let machine_id_path = host_dir.join("machine-id");
    std::fs::write(&machine_id_path, format!("{machine_id}\n"))?;

    let disk_info_path = host_dir.join("disk-serial");
    std::fs::write(&disk_info_path, format!("{}\n", hw.disk_serial))?;

    let disk_model_path = host_dir.join("disk-model");
    std::fs::write(&disk_model_path, format!("{}\n", hw.disk_model))?;

    let mut docker_args = Vec::new();

    for &(filename, _) in DMI_FILES {
        let host_path = dmi_dir.join(filename);
        let guest_path = format!("/sys/class/dmi/id/{filename}");
        docker_args.push("-v".to_string());
        docker_args.push(format!("{}:{}:ro", host_path.display(), guest_path));
    }

    docker_args.push("-v".to_string());
    docker_args.push(format!(
        "{}:/etc/machine-id:ro",
        machine_id_path.display()
    ));

    docker_args.push("-v".to_string());
    docker_args.push(format!(
        "{}:/run/spoof/machine-id:ro",
        machine_id_path.display()
    ));

    Ok(SpoofFiles {
        host_dir,
        machine_id,
        docker_args,
    })
}

pub fn mac_docker_arg(mac: &str) -> Vec<String> {
    vec!["--mac-address".to_string(), mac.to_string()]
}

pub fn cleanup_spoof_files(
    spoof_dir: &str,
    container_name: &str,
) -> Result<(), ContainerSpoofError> {
    let host_dir = Path::new(spoof_dir).join(container_name);
    if host_dir.exists() {
        std::fs::remove_dir_all(&host_dir)?;
    }
    Ok(())
}

pub fn verify_args(container_name: &str, hw: &HwIdentity) -> Vec<(&'static str, String, String)> {
    let _ = container_name;
    vec![
        (
            "machine-id",
            generate_machine_id(container_name),
            "cat /etc/machine-id".to_string(),
        ),
        (
            "sys_vendor",
            hw.smbios_manufacturer.clone(),
            "cat /sys/class/dmi/id/sys_vendor 2>/dev/null || echo N/A".to_string(),
        ),
        (
            "product_name",
            hw.smbios_product.clone(),
            "cat /sys/class/dmi/id/product_name 2>/dev/null || echo N/A".to_string(),
        ),
        (
            "product_serial",
            hw.smbios_serial.clone(),
            "cat /sys/class/dmi/id/product_serial 2>/dev/null || echo N/A".to_string(),
        ),
        (
            "mac_address",
            hw.mac_address.clone(),
            "ip link show eth0 | grep link/ether | awk '{print $2}'".to_string(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spoof;

    #[test]
    fn test_machine_id_deterministic() {
        let id1 = generate_machine_id("container-1");
        let id2 = generate_machine_id("container-1");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_machine_id_format() {
        let id = generate_machine_id("test-container");
        assert_eq!(id.len(), 32);
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_different_containers_different_ids() {
        let id1 = generate_machine_id("container-a");
        let id2 = generate_machine_id("container-b");
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_create_spoof_files() {
        let tmp = std::env::temp_dir().join("vmctl-spoof-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let hw = spoof::generate_identity("test-container");
        let result = create_spoof_files(tmp.to_str().unwrap(), "test-container", &hw);
        assert!(result.is_ok());

        let spoof_files = result.unwrap();
        assert!(spoof_files.host_dir.exists());
        assert!(!spoof_files.docker_args.is_empty());
        assert_eq!(spoof_files.machine_id.len(), 32);

        let dmi_dir = spoof_files.host_dir.join("dmi");
        assert!(dmi_dir.join("sys_vendor").exists());
        assert!(dmi_dir.join("product_name").exists());
        assert!(dmi_dir.join("product_serial").exists());

        let vendor = std::fs::read_to_string(dmi_dir.join("sys_vendor")).unwrap();
        assert_eq!(vendor.trim(), hw.smbios_manufacturer);

        let product = std::fs::read_to_string(dmi_dir.join("product_name")).unwrap();
        assert_eq!(product.trim(), hw.smbios_product);

        let mid = std::fs::read_to_string(spoof_files.host_dir.join("machine-id")).unwrap();
        assert_eq!(mid.trim(), spoof_files.machine_id);

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cleanup_spoof_files() {
        let tmp = std::env::temp_dir().join("vmctl-spoof-cleanup-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let hw = spoof::generate_identity("cleanup-test");
        create_spoof_files(tmp.to_str().unwrap(), "cleanup-test", &hw).unwrap();

        let dir = tmp.join("cleanup-test");
        assert!(dir.exists());

        cleanup_spoof_files(tmp.to_str().unwrap(), "cleanup-test").unwrap();
        assert!(!dir.exists());

        assert!(cleanup_spoof_files(tmp.to_str().unwrap(), "nonexistent").is_ok());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_mac_docker_arg() {
        let args = mac_docker_arg("a4:bb:6d:12:34:56");
        assert_eq!(args, vec!["--mac-address", "a4:bb:6d:12:34:56"]);
    }

    #[test]
    fn test_docker_args_contain_volume_mounts() {
        let tmp = std::env::temp_dir().join("vmctl-spoof-args-test");
        let _ = std::fs::remove_dir_all(&tmp);

        let hw = spoof::generate_identity("args-test");
        let spoof_files =
            create_spoof_files(tmp.to_str().unwrap(), "args-test", &hw).unwrap();

        assert!(spoof_files.docker_args.len() >= 4);
        let v_count = spoof_files.docker_args.iter().filter(|a| *a == "-v").count();
        assert!(v_count >= 2, "Expected at least 2 -v mounts, got {v_count}");

        let has_dmi = spoof_files
            .docker_args
            .iter()
            .any(|a| a.contains("/sys/class/dmi/id/"));
        assert!(has_dmi, "Docker args should include DMI bind mounts");

        let has_mid = spoof_files
            .docker_args
            .iter()
            .any(|a| a.contains("/etc/machine-id"));
        assert!(has_mid, "Docker args should include machine-id bind mount");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_verify_args() {
        let hw = spoof::generate_identity("verify-test");
        let checks = verify_args("verify-test", &hw);
        assert_eq!(checks.len(), 5);
        assert_eq!(checks[0].0, "machine-id");
        assert_eq!(checks[1].0, "sys_vendor");
        assert_eq!(checks[4].0, "mac_address");
    }
}
