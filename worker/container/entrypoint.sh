#!/bin/bash
set -e

if [ -f /run/spoof/machine-id ]; then
    if ! cmp -s /run/spoof/machine-id /etc/machine-id; then
        cat /run/spoof/machine-id > /etc/machine-id || true
    fi
fi

mkdir -p "${XDG_RUNTIME_DIR}"
chmod 700 "${XDG_RUNTIME_DIR}"
chown farmuser:farmuser "${XDG_RUNTIME_DIR}"

if [ -e /dev/dri/renderD128 ]; then
    chmod 666 /dev/dri/renderD128 || true
fi

echo "Starting environment as farmuser..."

exec su -l farmuser -c "
    export XDG_RUNTIME_DIR=${XDG_RUNTIME_DIR}
    export WAYLAND_DISPLAY=${WAYLAND_DISPLAY}
    export WLR_BACKENDS=${WLR_BACKENDS}
    export WLR_LIBINPUT_NO_DEVICES=${WLR_LIBINPUT_NO_DEVICES}
    export WLR_RENDERER=${WLR_RENDERER:-pixman}
    export MESA_VK_WSI_PRESENT_MODE=${MESA_VK_WSI_PRESENT_MODE}

    # Запускаем Steam launcher (он должен ждать появления сокета)
    /opt/farm/steam-launcher.sh &

    # Запускаем Sway через dbus
    echo 'Launching Sway...'
    exec dbus-run-session sway --debug 2>&1
"