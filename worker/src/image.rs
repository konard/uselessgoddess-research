//! Base image preparation and setup for CS2 farming VMs.
//!
//! Handles qcow2 base image management including:
//! - Cloud image download tracking
//! - NBD-based offline image customization
//! - cloud-init configuration generation
//! - Base image metadata

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageError {
    #[error("command failed ({cmd}): {reason}")]
    Command { cmd: String, reason: String },
    #[error("path not found: {0}")]
    NotFound(String),
    #[error("I/O error: {0}")]
    Io(String),
}

/// Describes the base image and its configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseImageConfig {
    /// Path to the qcow2 base image.
    pub image_path: String,
    /// OS type (e.g., "debian-12", "ubuntu-22.04").
    pub os_type: String,
    /// Packages to install during image preparation.
    pub packages: Vec<String>,
    /// Whether the image has been prepared (packages installed, services configured).
    pub prepared: bool,
}

impl Default for BaseImageConfig {
    fn default() -> Self {
        Self {
            image_path: "/var/lib/vmctl/base/cs2-farm-base.qcow2".into(),
            os_type: "debian-12".into(),
            packages: default_packages(),
            prepared: false,
        }
    }
}

/// Default packages required for CS2 farming VM.
fn default_packages() -> Vec<String> {
    [
        "qemu-guest-agent",
        "steam-installer",
        "mesa-vulkan-drivers",
        "xserver-xorg-core",
        "openbox",
        "xinit",
        "pulseaudio",
        "dmidecode",
        "lsblk",
        "curl",
        "ca-certificates",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// Generate cloud-init user-data YAML for automatic VM provisioning.
///
/// This creates a cloud-init config that:
/// - Sets up a farm user
/// - Installs required packages
/// - Enables qemu-guest-agent
/// - Configures auto-login to X11 + OpenBox
/// - Creates a systemd service for Steam auto-start
pub fn generate_cloud_init_userdata(farm_user: &str, farm_password: &str) -> String {
    format!(
        r#"#cloud-config
hostname: cs2-farm-vm
manage_etc_hosts: true

users:
  - name: {user}
    groups: [sudo, video, audio, render]
    shell: /bin/bash
    lock_passwd: false
    plain_text_passwd: "{password}"
    sudo: ALL=(ALL) NOPASSWD:ALL

package_update: true
packages:
  - qemu-guest-agent
  - steam-installer
  - mesa-vulkan-drivers
  - xserver-xorg-core
  - openbox
  - xinit
  - pulseaudio
  - dmidecode

runcmd:
  # Enable and start guest agent
  - systemctl enable qemu-guest-agent
  - systemctl start qemu-guest-agent

  # Create steam launcher script
  - |
    cat > /opt/farm/steam-launcher.sh << 'LAUNCHER'
    #!/bin/bash
    # Wait for session files to be injected via guest agent
    while [ ! -f /home/{user}/.steam/steam/config/.ready ]; do
        sleep 2
    done
    rm -f /home/{user}/.steam/steam/config/.ready

    # Launch Steam in silent mode with minimal UI
    exec steam -silent -no-browser -console \
        -w 384 -h 288 \
        +connect_lobby default
    LAUNCHER
  - mkdir -p /opt/farm
  - chmod +x /opt/farm/steam-launcher.sh
  - chown {user}:{user} /opt/farm/steam-launcher.sh

  # Create systemd service for Steam auto-launch
  - |
    cat > /etc/systemd/system/steam-farm.service << 'SERVICE'
    [Unit]
    Description=Steam CS2 Farming Session
    After=network.target display-manager.service

    [Service]
    Type=simple
    User={user}
    Environment=DISPLAY=:0
    Environment=DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus
    ExecStart=/opt/farm/steam-launcher.sh
    Restart=on-failure
    RestartSec=10

    [Install]
    WantedBy=multi-user.target
    SERVICE
  - systemctl daemon-reload
  - systemctl enable steam-farm.service

  # Auto-start X11 with OpenBox on boot
  - |
    cat > /etc/systemd/system/x11-auto.service << 'XSERVICE'
    [Unit]
    Description=Auto-start X11 with OpenBox
    After=systemd-user-sessions.service

    [Service]
    Type=simple
    User={user}
    ExecStart=/usr/bin/startx /usr/bin/openbox-session -- :0 vt7
    Restart=on-failure
    RestartSec=5

    [Install]
    WantedBy=graphical.target
    XSERVICE
  - systemctl daemon-reload
  - systemctl enable x11-auto.service

  # Create virtiofs mount point for shared CS2 installation
  - mkdir -p /opt/cs2
  - |
    echo 'cs2 /opt/cs2 virtiofs defaults,nofail 0 0' >> /etc/fstab

final_message: "CS2 farm VM provisioning complete"
"#,
        user = farm_user,
        password = farm_password,
    )
}

/// Generate cloud-init meta-data for a specific VM instance.
pub fn generate_cloud_init_metadata(vm_name: &str) -> String {
    format!(
        r#"instance-id: {name}
local-hostname: {name}
"#,
        name = vm_name
    )
}

/// Create a cloud-init ISO (seed image) for VM provisioning.
///
/// Generates user-data and meta-data files and packs them into an ISO
/// using `genisoimage` or `mkisofs`.
pub fn create_cloud_init_iso(
    output_path: &str,
    vm_name: &str,
    farm_user: &str,
    farm_password: &str,
) -> Result<(), ImageError> {
    let tmp_dir = format!("/tmp/vmctl-cloud-init-{vm_name}");
    std::fs::create_dir_all(&tmp_dir).map_err(|e| ImageError::Io(e.to_string()))?;

    let userdata = generate_cloud_init_userdata(farm_user, farm_password);
    let metadata = generate_cloud_init_metadata(vm_name);

    std::fs::write(format!("{tmp_dir}/user-data"), userdata)
        .map_err(|e| ImageError::Io(e.to_string()))?;
    std::fs::write(format!("{tmp_dir}/meta-data"), metadata)
        .map_err(|e| ImageError::Io(e.to_string()))?;

    // Try genisoimage first, fall back to mkisofs
    let iso_cmd = if which_exists("genisoimage") {
        "genisoimage"
    } else {
        "mkisofs"
    };

    let output = Command::new(iso_cmd)
        .args([
            "-output",
            output_path,
            "-volid",
            "cidata",
            "-joliet",
            "-rock",
            &format!("{tmp_dir}/user-data"),
            &format!("{tmp_dir}/meta-data"),
        ])
        .output()
        .map_err(|e| ImageError::Command {
            cmd: iso_cmd.to_string(),
            reason: e.to_string(),
        })?;

    // Cleanup temp files
    let _ = std::fs::remove_dir_all(&tmp_dir);

    if !output.status.success() {
        return Err(ImageError::Command {
            cmd: iso_cmd.to_string(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

/// Resize a qcow2 image.
pub fn resize_image(image_path: &str, size: &str) -> Result<(), ImageError> {
    if !Path::new(image_path).exists() {
        return Err(ImageError::NotFound(image_path.to_string()));
    }

    let output = Command::new("qemu-img")
        .args(["resize", image_path, size])
        .output()
        .map_err(|e| ImageError::Command {
            cmd: "qemu-img resize".into(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ImageError::Command {
            cmd: "qemu-img resize".into(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

/// Convert an image to compressed qcow2 format.
pub fn compress_image(input_path: &str, output_path: &str) -> Result<(), ImageError> {
    if !Path::new(input_path).exists() {
        return Err(ImageError::NotFound(input_path.to_string()));
    }

    let output = Command::new("qemu-img")
        .args([
            "convert", "-c", "-O", "qcow2", input_path, output_path,
        ])
        .output()
        .map_err(|e| ImageError::Command {
            cmd: "qemu-img convert".into(),
            reason: e.to_string(),
        })?;

    if !output.status.success() {
        return Err(ImageError::Command {
            cmd: "qemu-img convert".into(),
            reason: String::from_utf8_lossy(&output.stderr).to_string(),
        });
    }
    Ok(())
}

fn which_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_packages() {
        let pkgs = default_packages();
        assert!(pkgs.contains(&"qemu-guest-agent".to_string()));
        assert!(pkgs.contains(&"steam-installer".to_string()));
        assert!(pkgs.contains(&"mesa-vulkan-drivers".to_string()));
        assert!(pkgs.contains(&"openbox".to_string()));
        assert!(pkgs.contains(&"dmidecode".to_string()));
    }

    #[test]
    fn test_cloud_init_userdata_contains_user() {
        let ud = generate_cloud_init_userdata("farmuser", "secret123");
        assert!(ud.contains("name: farmuser"));
        assert!(ud.contains("plain_text_passwd: \"secret123\""));
        assert!(ud.contains("qemu-guest-agent"));
        assert!(ud.contains("steam-farm.service"));
        assert!(ud.contains("steam-launcher.sh"));
    }

    #[test]
    fn test_cloud_init_userdata_contains_virtiofs_mount() {
        let ud = generate_cloud_init_userdata("farmuser", "pass");
        assert!(ud.contains("/opt/cs2"));
        assert!(ud.contains("virtiofs"));
    }

    #[test]
    fn test_cloud_init_metadata() {
        let md = generate_cloud_init_metadata("cs2-farm-0");
        assert!(md.contains("instance-id: cs2-farm-0"));
        assert!(md.contains("local-hostname: cs2-farm-0"));
    }

    #[test]
    fn test_base_image_config_default() {
        let cfg = BaseImageConfig::default();
        assert!(!cfg.prepared);
        assert_eq!(cfg.os_type, "debian-12");
        assert!(!cfg.packages.is_empty());
    }

    #[test]
    fn test_base_image_config_serialization() {
        let cfg = BaseImageConfig::default();
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: BaseImageConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.os_type, cfg.os_type);
        assert_eq!(parsed.packages.len(), cfg.packages.len());
    }

    #[test]
    fn test_resize_not_found() {
        let err = resize_image("/nonexistent/image.qcow2", "30G");
        assert!(matches!(err, Err(ImageError::NotFound(_))));
    }

    #[test]
    fn test_compress_not_found() {
        let err = compress_image("/nonexistent/input.qcow2", "/tmp/output.qcow2");
        assert!(matches!(err, Err(ImageError::NotFound(_))));
    }

    #[test]
    fn test_cloud_init_userdata_is_valid_yaml_header() {
        let ud = generate_cloud_init_userdata("user", "pass");
        assert!(ud.starts_with("#cloud-config\n"));
    }
}
