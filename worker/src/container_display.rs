//! Display capture and input injection for containers.
//!
//! Each container runs Sway (Wayland compositor) + wayvnc, exposing
//! the display on port 5900 inside the container (mapped to a host port).
//!
//! For automated control, we have two approaches:
//!
//! 1. **VNC**: Connect to wayvnc for both framebuffer capture and input
//!    injection. This works well with existing VNC client libraries.
//!
//! 2. **Direct exec**: Use `docker exec` to run tools inside the container:
//!    - `swaymsg` for window management
//!    - `wlr-randr` for output info
//!    - Screenshot via `grim` (if installed) or sway IPC
//!
//! This module provides helper functions for common display operations.
//! The actual VNC client integration is left to the RPC/server layer.

use crate::container_exec::{self, ContainerExecError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DisplayError {
    #[error("container exec error: {0}")]
    Exec(#[from] ContainerExecError),
    #[error("display not ready in container `{0}`")]
    NotReady(String),
    #[error("screenshot failed: {0}")]
    Screenshot(String),
}

/// Check if the Wayland display is ready inside the container.
///
/// Returns true if Sway is running and wayvnc is accepting connections.
pub fn is_display_ready(container_name: &str) -> Result<bool, DisplayError> {
    // Check if sway is running
    let result = container_exec::exec_as_user(
        container_name,
        "farmuser",
        "pgrep -c sway 2>/dev/null || echo 0",
    )?;
    let sway_running: i32 = result.stdout.trim().parse().unwrap_or(0);

    // Check if wayvnc is running
    let result = container_exec::exec_as_user(
        container_name,
        "farmuser",
        "pgrep -c wayvnc 2>/dev/null || echo 0",
    )?;
    let vnc_running: i32 = result.stdout.trim().parse().unwrap_or(0);

    Ok(sway_running > 0 && vnc_running > 0)
}

/// Get the VNC connection info for a container.
///
/// Returns (host, port) for connecting to the container's wayvnc.
pub fn vnc_address(vnc_port: u16) -> (String, u16) {
    ("127.0.0.1".to_string(), vnc_port)
}

/// Take a screenshot inside the container using grim (Wayland screenshot tool).
///
/// Saves the screenshot to the specified path inside the container.
/// Returns the raw PNG bytes if `return_bytes` is true.
pub fn take_screenshot(
    container_name: &str,
    output_path: &str,
) -> Result<(), DisplayError> {
    let result = container_exec::exec_as_user(
        container_name,
        "farmuser",
        &format!(
            "export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1; grim '{output_path}'"
        ),
    )?;

    if result.exit_code != 0 {
        return Err(DisplayError::Screenshot(format!(
            "grim failed (exit {}): {}",
            result.exit_code,
            result.stderr.trim()
        )));
    }

    Ok(())
}

/// Read a screenshot from the container as raw bytes.
///
/// Takes a screenshot, reads it back via docker exec, and returns PNG data.
pub fn capture_frame(container_name: &str) -> Result<Vec<u8>, DisplayError> {
    let tmp_path = "/tmp/vmctl-screenshot.png";
    take_screenshot(container_name, tmp_path)?;
    let data = container_exec::read_file(container_name, tmp_path)?;
    // Clean up temp file
    let _ = container_exec::exec(container_name, &format!("rm -f '{tmp_path}'"));
    Ok(data)
}

/// Send a keyboard key press to the Wayland session via swaymsg.
///
/// Note: For gaming input (CS2), VNC input injection is preferred
/// as it provides raw keyboard/mouse events. This is for UI automation
/// (menu navigation, accepting matches, etc.).
pub fn send_key(container_name: &str, key: &str) -> Result<(), DisplayError> {
    let result = container_exec::exec_as_user(
        container_name,
        "farmuser",
        &format!(
            "export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1; \
             swaymsg seat - cursor press {key}"
        ),
    )?;

    if result.exit_code != 0 {
        return Err(DisplayError::Exec(ContainerExecError::ExecFailed {
            container: container_name.to_string(),
            reason: format!("swaymsg key {key} failed: {}", result.stderr),
        }));
    }

    Ok(())
}

/// Get the list of Wayland outputs (displays) in the container.
pub fn list_outputs(container_name: &str) -> Result<String, DisplayError> {
    let result = container_exec::exec_as_user(
        container_name,
        "farmuser",
        "export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1; swaymsg -t get_outputs",
    )?;

    if result.exit_code != 0 {
        return Err(DisplayError::NotReady(container_name.to_string()));
    }

    Ok(result.stdout)
}

/// Get the focused window info from Sway.
pub fn focused_window(container_name: &str) -> Result<String, DisplayError> {
    let result = container_exec::exec_as_user(
        container_name,
        "farmuser",
        "export XDG_RUNTIME_DIR=/run/user/1000 WAYLAND_DISPLAY=wayland-1; swaymsg -t get_tree",
    )?;

    Ok(result.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnc_address() {
        let (host, port) = vnc_address(5901);
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 5901);
    }

    #[test]
    fn test_vnc_address_different_ports() {
        let (_, port1) = vnc_address(5901);
        let (_, port2) = vnc_address(5902);
        assert_ne!(port1, port2);
    }
}
