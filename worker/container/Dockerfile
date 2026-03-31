# CS2 Farm Container — Wayland (Sway) + Mesa Vulkan + Steam
#
# Base: Ubuntu 24.04 (latest LTS, fresh Mesa/Vulkan, systemd-free for containers)
# GPU: AMD/Intel via /dev/dri passthrough (render nodes)
# Display: Sway (Wayland compositor) + wayvnc for remote access
# Audio: None (CS2 runs with -nosound)
#
# Build:
#   docker build -t cs2-farm:latest -f container/Dockerfile container/
#
# Run (example):
#   docker run -d --name cs2-farm-0 \
#     --device /dev/dri:/dev/dri \
#     -v /opt/cs2-shared:/opt/cs2:ro \
#     -p 5900:5900 \
#     cs2-farm:latest

FROM ubuntu:24.04

ARG DEBIAN_FRONTEND=noninteractive

# --- Layer 1: Base system + Mesa/Vulkan (AMD & Intel) ---
RUN apt-get update && apt-get install -y --no-install-recommends \
    # Mesa Vulkan drivers (AMD radv, Intel anv, software lavapipe)
    mesa-vulkan-drivers \
    libvulkan1 \
    vulkan-tools \
    libegl1 \
    libgles2 \
    libgl1-mesa-dri \
    libglx-mesa0 \
    # DRM/DRI userspace
    libdrm2 \
    libdrm-amdgpu1 \
    libdrm-intel1 \
    # Wayland compositor
    sway \
    # VNC server for Wayland
    wayvnc \
    # XWayland for Steam/CS2 (Steam still uses X11 internally)
    xwayland \
    # Terminal (lightweight, for debugging)
    foot \
    # Core utils
    dbus-user-session \
    procps \
    iproute2 \
    curl \
    ca-certificates \
    # Hardware info tools (for spoofing verification)
    dmidecode \
    pciutils \
    # Libs needed by Steam
    libc6:amd64 \
    libstdc++6:amd64 \
    libx11-6 \
    libxext6 \
    libxrandr2 \
    libxrender1 \
    libxfixes3 \
    libxcursor1 \
    libxi6 \
    libxtst6 \
    libpulse0 \
    libnss3 \
    libnspr4 \
    libdbus-1-3 \
    libglib2.0-0 \
    libcairo2 \
    libpango-1.0-0 \
    libpangocairo-1.0-0 \
    libatk1.0-0 \
    libgdk-pixbuf-2.0-0 \
    libfontconfig1 \
    libfreetype6 \
    && rm -rf /var/lib/apt/lists/*

# Enable i386 architecture for Steam 32-bit dependencies
RUN dpkg --add-architecture i386 && apt-get update && apt-get install -y --no-install-recommends \
    libc6:i386 \
    libstdc++6:i386 \
    libgl1:i386 \
    libx11-6:i386 \
    libxext6:i386 \
    libxrandr2:i386 \
    libxrender1:i386 \
    libxfixes3:i386 \
    libpulse0:i386 \
    libnss3:i386 \
    libnspr4:i386 \
    libdbus-1-3:i386 \
    libglib2.0-0:i386 \
    libcairo2:i386 \
    libpango-1.0-0:i386 \
    libpangocairo-1.0-0:i386 \
    libatk1.0-0:i386 \
    libgdk-pixbuf-2.0-0:i386 \
    libfontconfig1:i386 \
    libfreetype6:i386 \
    && rm -rf /var/lib/apt/lists/*

# --- Layer 2: Farm user ---
RUN groupadd -f render && \
    useradd -m -s /bin/bash -G video,render farmuser

# --- Layer 3: Sway configuration ---
RUN mkdir -p /home/farmuser/.config/sway && \
    chown -R farmuser:farmuser /home/farmuser/.config

COPY sway-config /home/farmuser/.config/sway/config
RUN chown farmuser:farmuser /home/farmuser/.config/sway/config

# --- Layer 4: Steam launcher + entrypoint scripts ---
RUN mkdir -p /opt/farm

COPY steam-launcher.sh /opt/farm/steam-launcher.sh
COPY entrypoint.sh /opt/farm/entrypoint.sh
RUN chmod +x /opt/farm/steam-launcher.sh /opt/farm/entrypoint.sh

# --- Layer 5: Steam directories (pre-create for bind mounts) ---
RUN mkdir -p /home/farmuser/.steam/steam/config \
    /home/farmuser/.steam/steam/ubuntu12_32 \
    /home/farmuser/.local/share/Steam && \
    chown -R farmuser:farmuser /home/farmuser/.steam /home/farmuser/.local

# CS2 shared mount point
RUN mkdir -p /opt/cs2

# Spoofing mount points (will be bind-mounted at runtime)
RUN mkdir -p /run/spoof

# Environment
ENV XDG_RUNTIME_DIR=/run/user/1000
ENV WAYLAND_DISPLAY=wayland-1
ENV WLR_BACKENDS=headless
ENV WLR_LIBINPUT_NO_DEVICES=1
ENV WLR_RENDERER=pixman
ENV MESA_VK_WSI_PRESENT_MODE=immediate

EXPOSE 5900

ENTRYPOINT ["/opt/farm/entrypoint.sh"]