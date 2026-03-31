use crate::container::exec as container_exec;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SteamLibraryError {
    #[error("container exec error: {0}")]
    Exec(#[from] container_exec::ContainerExecError),
}

/// Default Steam library folders path inside the container.
const STEAM_LIBRARY_VDF_PATH: &str = "/home/farmuser/.steam/steam/config/libraryfolders.vdf";

/// Generate a `libraryfolders.vdf` that includes the shared CS2 directory.
///
/// Steam uses this file to know where to look for installed games.
/// By adding `/opt/cs2` (the container mount point for `/opt/cs2-shared`),
/// Steam will recognize CS2 as already installed without re-downloading.
pub fn generate_libraryfolders_vdf(cs2_mount_path: &str) -> String {
    format!(
        r#""libraryfolders"
{{
	"0"
	{{
		"path"		"/home/farmuser/.steam/steam"
		"label"		""
		"contentid"		""
		"totalsize"		"0"
		"apps"
		{{
		}}
	}}
	"1"
	{{
		"path"		"{cs2_mount_path}"
		"label"		"CS2 Shared"
		"contentid"		""
		"totalsize"		"0"
		"apps"
		{{
			"730"		"0"
		}}
	}}
}}"#
    )
}

/// Inject Steam library folders configuration into a running container.
///
/// This adds `/opt/cs2` (the bind-mounted CS2 shared directory) as a
/// Steam library folder so that Steam recognizes CS2 as installed.
pub fn inject_library_folders(
    container_name: &str,
    cs2_mount_path: Option<&str>,
) -> Result<(), SteamLibraryError> {
    let mount_path = cs2_mount_path.unwrap_or("/opt/cs2");

    let vdf = generate_libraryfolders_vdf(mount_path);

    container_exec::write_file(container_name, STEAM_LIBRARY_VDF_PATH, vdf.as_bytes())?;

    // Ensure correct ownership
    container_exec::exec(
        container_name,
        &format!(
            "chown farmuser:farmuser '{STEAM_LIBRARY_VDF_PATH}'"
        ),
    )?;

    // Create the steamapps directory in the CS2 mount path if writable,
    // or create a symlink so Steam finds the appmanifest
    let steamapps_dir = format!("{mount_path}/steamapps");
    let result = container_exec::exec(
        container_name,
        &format!("test -d '{steamapps_dir}' && echo exists || echo missing"),
    )?;

    if result.stdout.trim() == "missing" {
        // The mount is read-only, so create a steamapps symlink structure
        // that points to the actual game data
        let _ = container_exec::exec(
            container_name,
            &format!(
                "mkdir -p '/home/farmuser/.local/share/Steam/steamapps' && \
                 test -d '{mount_path}/common' && \
                 ln -sfn '{mount_path}/common' '/home/farmuser/.local/share/Steam/steamapps/common' || true"
            ),
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_libraryfolders_vdf() {
        let vdf = generate_libraryfolders_vdf("/opt/cs2");
        assert!(vdf.contains("\"libraryfolders\""));
        assert!(vdf.contains("\"/opt/cs2\""));
        assert!(vdf.contains("\"730\""));
        assert!(vdf.contains("\"CS2 Shared\""));
    }

    #[test]
    fn test_generate_libraryfolders_vdf_custom_path() {
        let vdf = generate_libraryfolders_vdf("/mnt/games/cs2");
        assert!(vdf.contains("\"/mnt/games/cs2\""));
    }

    #[test]
    fn test_vdf_has_default_steam_library() {
        let vdf = generate_libraryfolders_vdf("/opt/cs2");
        assert!(vdf.contains("\"/home/farmuser/.steam/steam\""));
    }

    #[test]
    fn test_default_vdf_path() {
        assert!(STEAM_LIBRARY_VDF_PATH.contains("libraryfolders.vdf"));
        assert!(STEAM_LIBRARY_VDF_PATH.starts_with("/home/farmuser"));
    }
}
