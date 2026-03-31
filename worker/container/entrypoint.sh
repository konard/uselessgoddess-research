#!/bin/bash
# Container entrypoint for CS2 farming.
#
# Sets up the runtime environment:
# 1. Creates XDG_RUNTIME_DIR
# 2. Applies spoofed machine-id if provided
# 3. Starts Sway (Wayland compositor)
# 4. Launches steam-launcher in background
#
# The host worker controls this container via `docker exec`.

set -e

# Create XDG runtime directory
mkdir -p "${XDG_RUNTIME_DIR}"
chmod 700 "${XDG_RUNTIME_DIR}"
chown farmuser:farmuser "${XDG_RUNTIME_DIR}"

# Apply spoofed machine-id if mounted
if [ -f /run/spoof/machine-id ]; then
    cp /run/spoof/machine-id /etc/machine-id
fi

# If GPU render node exists, ensure farmuser can access it
if [ -e /dev/dri/renderD128 ]; then
    chmod 666 /dev/dri/renderD128 2>/dev/null || true
fi

# Start Sway as farmuser (Wayland compositor + wayvnc)
# Then start the Steam launcher in background
exec su -l farmuser -c "
    export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR}
    export WAYLAND_DISPLAY=${WAYLAND_DISPLAY}
    export WLR_BACKENDS=${WLR_BACKENDS}
    export WLR_LIBINPUT_NO_DEVICES=${WLR_LIBINPUT_NO_DEVICES}
    export WLR_RENDERER=${WLR_RENDERER:-pixman}
    export MESA_VK_WSI_PRESENT_MODE=${MESA_VK_WSI_PRESENT_MODE}

    # Start steam-launcher in background (waits for .ready marker)
    /opt/farm/steam-launcher.sh &

    # Start Sway (foreground — keeps container alive)
    exec sway
"
