# RUN_INSTRUCTIONS — worker

## Prerequisites

### Hardware
- x86_64 CPU
- AMD or Intel GPU with Vulkan support (via `/dev/dri/renderD128`)
- Recommended: 16+ GB RAM for 4+ containers (each uses ~2 GB)

### Software (host)
| Package | Purpose |
|---------|---------|
| Docker | Container runtime |
| GPU driver | AMD (amdgpu) or Intel (i915) with render nodes |
| steamcmd | CS2 updates (optional) |

### Install dependencies

**Ubuntu / Debian:**
```bash
# Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $(whoami)
# Log out and back in

# GPU driver (usually already installed)
# AMD: sudo apt install mesa-vulkan-drivers
# Intel: sudo apt install mesa-vulkan-drivers
```

## Build

```bash
cd worker
cargo build --release

# Build the Docker image
docker build -t cs2-farm:latest -f container/Dockerfile container/
```

## Usage

### 1. Check host dependencies
```bash
./target/release/worker check-deps
```

### 2. Create a single container
```bash
./target/release/worker create \
  --name cs2-farm-0 \
  --ram 2g \
  --cpus 2.0 \
  --vnc-port 5901 \
  --cs2-shared-dir /opt/cs2-shared
```

### 3. Batch-create multiple containers
```bash
./target/release/worker setup \
  --count 4 \
  --prefix cs2-farm \
  --ram 2g \
  --cpus 2.0 \
  --vnc-start 5901 \
  --cs2-shared-dir /opt/cs2-shared
```

### 4. Container lifecycle
```bash
./target/release/worker list
./target/release/worker start cs2-farm-0
./target/release/worker stop cs2-farm-0
./target/release/worker destroy cs2-farm-0
```

### 5. Execute commands inside container
```bash
./target/release/worker exec cs2-farm-0 --cmd "uname -a"
./target/release/worker exec cs2-farm-0 --cmd "vulkaninfo --summary"
./target/release/worker exec cs2-farm-0 --cmd "cat /etc/machine-id"
```

### 6. Verify hardware spoofing
```bash
./target/release/worker verify cs2-farm-0
./target/release/worker verify cs2-farm-0 --json
```

### 7. Show spoofed identity
```bash
./target/release/worker show-identity cs2-farm-0
```

### 8. Steam session injection
```bash
./target/release/worker inject-session cs2-farm-0 \
  --account mysteamuser \
  --token "eyJhbGciOi..." \
  --steam-id 76561198012345678 \
  --persona "FarmBot"

# Switch to different account
./target/release/worker switch-account cs2-farm-0 \
  --account otheruser \
  --token "eyJhbGciOi..." \
  --steam-id 76561198087654321 \
  --persona "FarmBot2"
```

### 9. CS2 updates
```bash
./target/release/worker cs2-status
./target/release/worker cs2-update \
  --shared-dir /opt/cs2-shared \
  --containers cs2-farm-0 --containers cs2-farm-1
```

### 10. Display management
```bash
# Check if Wayland/VNC is ready
./target/release/worker display-status cs2-farm-0

# Take a screenshot
./target/release/worker screenshot cs2-farm-0 --output /tmp/screenshot.png

# Connect via VNC (manual)
vncviewer localhost:5901
```

## Architecture

```
Host (Linux + Docker + GPU driver)
│
├── worker binary
│   ├── create          → docker run + spoof files + GPU + VNC
│   ├── setup           → batch create N containers
│   ├── exec            → docker exec (command execution)
│   ├── verify          → check spoofing via docker exec
│   ├── inject-session  → write Steam session via docker exec
│   └── cs2-update      → steamcmd on host, notify containers
│
├── /var/lib/vmctl/container-spoof/{name}/
│   ├── dmi/sys_vendor, product_name, product_serial, ...
│   └── machine-id
│
├── /opt/cs2-shared/          → CS2 installation (bind-mounted read-only)
│
└── Docker containers (each):
    ├── Ubuntu 24.04 + Mesa Vulkan (AMD radv / Intel anv)
    ├── Sway (Wayland compositor, headless)
    ├── wayvnc (VNC server on :5900)
    ├── XWayland (for Steam/CS2)
    ├── /dev/dri (GPU render nodes from host)
    ├── /sys/class/dmi/id/* (spoofed, bind-mounted)
    ├── /etc/machine-id (spoofed, bind-mounted)
    ├── --mac-address (spoofed by Docker)
    └── /opt/cs2 (CS2 shared mount, read-only)
```

## Hardware Spoofing

| What | How |
|------|-----|
| MAC address | `--mac-address` flag with real vendor OUI |
| SMBIOS/DMI | Bind-mount fake files over `/sys/class/dmi/id/` |
| Machine-ID | Bind-mount fake `/etc/machine-id` |
| No hypervisor artifacts | Containers share host kernel (no CPUID hypervisor bit) |

## Next Steps (beyond this PoC)

1. **VNC controller**: Read display buffer, inject keyboard/mouse from host for gameplay automation
2. **shai RPC integration**: Connect to central server for account queue, heartbeat, farm coordination
