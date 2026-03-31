#!/bin/bash
# Steam launcher for CS2 farming container.
# Waits for session files to be injected, then launches Steam + CS2.

STEAM_CONFIG_DIR="/home/farmuser/.steam/steam/config"
CS2_MOUNT="/opt/cs2"

echo "[steam-launcher] Waiting for session injection (.ready marker)..."

while [ ! -f "${STEAM_CONFIG_DIR}/.ready" ]; do
    sleep 1
done

rm -f "${STEAM_CONFIG_DIR}/.ready"
echo "[steam-launcher] Session detected."

# Configure /opt/cs2 as a Steam library folder if it exists and has game data.
# This prevents Steam from re-downloading CS2 by recognizing the shared install.
if [ -d "${CS2_MOUNT}" ]; then
    echo "[steam-launcher] Configuring ${CS2_MOUNT} as Steam library folder..."

    # Ensure libraryfolders.vdf includes our CS2 mount.
    # The worker injects this file, but if missing, create a fallback.
    if [ ! -f "${STEAM_CONFIG_DIR}/libraryfolders.vdf" ]; then
        cat > "${STEAM_CONFIG_DIR}/libraryfolders.vdf" <<'VDFEOF'
"libraryfolders"
{
	"0"
	{
		"path"		"/home/farmuser/.steam/steam"
		"label"		""
		"contentid"		""
		"totalsize"		"0"
		"apps"
		{
		}
	}
	"1"
	{
		"path"		"/opt/cs2"
		"label"		"CS2 Shared"
		"contentid"		""
		"totalsize"		"0"
		"apps"
		{
			"730"		"0"
		}
	}
}
VDFEOF
        echo "[steam-launcher] Created fallback libraryfolders.vdf"
    fi

    # Create steamapps structure if the mount has common/ dir
    if [ -d "${CS2_MOUNT}/common" ] && [ ! -d "${CS2_MOUNT}/steamapps" ]; then
        mkdir -p "/home/farmuser/.local/share/Steam/steamapps"
        ln -sfn "${CS2_MOUNT}/common" "/home/farmuser/.local/share/Steam/steamapps/common" 2>/dev/null || true
        echo "[steam-launcher] Linked CS2 game data from shared mount"
    fi
fi

echo "[steam-launcher] Launching Steam..."

# Use disk-backed tmp to avoid OOM during Steam operations
export TMPDIR=/var/tmp

exec steam \
    -silent \
    -no-browser \
    -console \
    -applaunch 730 \
    -nosound \
    -novid \
    -windowed \
    -w 384 \
    -h 288 \
    +connect_lobby default
