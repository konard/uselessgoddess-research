# RUN_INSTRUCTIONS — vmctl Worker PoC

The worker supports two modes:
- **VM mode**: Full KVM/QEMU virtual machines (original)
- **Container mode**: Lightweight Docker containers (new, recommended for development)

---

# Container Mode (Recommended)

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
cd container
docker build -t cs2-farm:latest .
cd ..
```

## Usage (Container Mode)

### 1. Check host dependencies
```bash
./target/release/worker container-check-deps
```

### 2. Create a single container
```bash
./target/release/worker container-create \
  --name cs2-farm-0 \
  --ram 2g \
  --cpus 2.0 \
  --vnc-port 5901 \
  --cs2-shared-dir /opt/cs2-shared
```

### 3. Batch-create multiple containers
```bash
./target/release/worker container-setup \
  --count 4 \
  --prefix cs2-farm \
  --ram 2g \
  --cpus 2.0 \
  --vnc-start 5901 \
  --cs2-shared-dir /opt/cs2-shared
```

### 4. Container lifecycle
```bash
./target/release/worker container-list
./target/release/worker container-start cs2-farm-0
./target/release/worker container-stop cs2-farm-0
./target/release/worker container-destroy cs2-farm-0
```

### 5. Execute commands inside container
```bash
./target/release/worker container-exec cs2-farm-0 --cmd "uname -a"
./target/release/worker container-exec cs2-farm-0 --cmd "vulkaninfo --summary"
./target/release/worker container-exec cs2-farm-0 --cmd "cat /etc/machine-id"
```

### 6. Verify hardware spoofing
```bash
./target/release/worker container-verify cs2-farm-0
./target/release/worker container-verify cs2-farm-0 --json
```

### 7. Show spoofed identity
```bash
./target/release/worker container-show-identity cs2-farm-0
```

### 8. Steam session injection
```bash
./target/release/worker container-inject-session cs2-farm-0 \
  --account mysteamuser \
  --token "eyJhbGciOi..." \
  --steam-id 76561198012345678 \
  --persona "FarmBot"

# Switch to different account
./target/release/worker container-switch-account cs2-farm-0 \
  --account otheruser \
  --token "eyJhbGciOi..." \
  --steam-id 76561198087654321 \
  --persona "FarmBot2"
```

### 9. CS2 updates
```bash
./target/release/worker cs2-status
./target/release/worker container-cs2-update \
  --shared-dir /opt/cs2-shared \
  --containers cs2-farm-0 --containers cs2-farm-1
```

### 10. Display management
```bash
# Check if Wayland/VNC is ready
./target/release/worker container-display-status cs2-farm-0

# Take a screenshot
./target/release/worker container-screenshot cs2-farm-0 --output /tmp/screenshot.png

# Connect via VNC (manual)
vncviewer localhost:5901
```

## Container Architecture

```
Host (Linux + Docker + GPU driver)
│
├── worker binary (this tool)
│   ├── container-create    → docker run + spoof files + GPU + VNC
│   ├── container-setup     → batch create N containers
│   ├── container-exec      → docker exec (replaces Guest Agent)
│   ├── container-verify    → check spoofing via docker exec
│   ├── container-inject    → write Steam session via docker exec
│   └── container-cs2-update → steamcmd on host, notify containers
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

### Container Spoofing (vs VM)

| Aspect | VM (KVM/QEMU) | Container (Docker) |
|--------|---------------|-------------------|
| MAC address | libvirt XML | `--mac-address` flag |
| SMBIOS/DMI | XML sysinfo block | Bind-mount fake /sys/class/dmi/id/ files |
| Machine-ID | cloud-init | Bind-mount fake /etc/machine-id |
| Disk serial | XML `<serial>` | N/A (container uses overlay FS) |
| CPU info | CPUID hiding | Host kernel (no hypervisor bit) |
| GPU | VirtIO-GPU Venus (~43% perf) | /dev/dri passthrough (native perf) |
| Hypervisor detection | Must hide KVM | Not applicable (shares host kernel) |

### Container Advantages over VM

1. **No hypervisor overhead**: Containers share the host kernel, no CPUID/timing artifacts
2. **Native GPU performance**: /dev/dri passthrough vs VirtIO-GPU Venus (~43% overhead)
3. **Lower RAM usage**: No guest OS overhead (~150 MB vs ~800 MB per instance)
4. **Faster startup**: Seconds vs minutes for VM boot + cloud-init
5. **Simpler setup**: No KVM, libvirt, QEMU, virtiofsd required
6. **Better disk efficiency**: Docker layers with CoW vs qcow2 overlays

---

# VM Mode (Original)

## Prerequisites

### Hardware
- x86_64 CPU with Intel VT-x or AMD-V
- Recommended: 32 GB RAM for 8+ VMs (each uses ~2 GB, ~1.5 GB effective with KSM)
- GPU for VirtIO-GPU Venus (Mesa 24.2+, kernel 6.13+)

### Software (host)
| Package | Min Version | Purpose |
|---------|-------------|---------|
| QEMU | 9.2+ | VM hypervisor with Venus GPU support |
| libvirt + virsh | any recent | VM lifecycle management |
| qemu-img | (matches QEMU) | Disk overlay creation |
| virtiofsd | any | Shared filesystem (CS2 binaries) |
| KVM kernel module | loaded | Hardware virtualization |
| steamcmd | any | CS2 updates (optional) |
| genisoimage/mkisofs | any | Cloud-init ISO creation |

### Install dependencies

**Ubuntu / Debian:**
```bash
sudo apt-get install -y qemu-system-x86 qemu-utils libvirt-daemon-system virtinst virtiofsd genisoimage
sudo systemctl enable --now libvirtd
sudo modprobe kvm kvm_intel  # or kvm_amd
```

**Gentoo:**
```bash
sudo emerge -av app-emulation/qemu app-emulation/libvirt app-emulation/virtiofsd dev-libs/cdrtools
sudo rc-update add libvirtd default
sudo rc-service libvirtd start
sudo modprobe kvm kvm_intel  # or kvm_amd
```

**Arch Linux:**
```bash
sudo pacman -S qemu-full libvirt virtiofsd cdrtools
sudo systemctl enable --now libvirtd
```

### User permissions
```bash
sudo usermod -aG libvirt,kvm $(whoami)
# Log out and back in for group changes to take effect
```

## Build

```bash
cd worker
cargo build --release
# Binary: target/release/worker
```

## Usage

### 1. Check host dependencies
```bash
./target/release/worker check-deps
```

### 2. Prepare a base disk image

Download a cloud image and create a cloud-init ISO for automatic provisioning:

```bash
# Download base image
mkdir -p /var/lib/vmctl/base
wget https://cloud-images.ubuntu.com/jammy/current/jammy-server-cloudimg-amd64.img \
  -O /var/lib/vmctl/base/ubuntu-22.04.qcow2

# Generate cloud-init ISO for automatic VM provisioning
# This installs qemu-guest-agent, Steam, mesa/Vulkan, sway, wayvnc, and systemd services
./target/release/worker cloud-init \
  --output /var/lib/vmctl/base/cloud-init.iso \
  --vm-name cs2-farm-0 \
  --user farmuser \
  --password farmpass
```

The cloud-init ISO auto-configures:
- Farm user with sudo access
- `qemu-guest-agent` (for host-to-guest communication)
- Steam + mesa Vulkan drivers (silent install via debconf pre-seeding)
- Sway (Wayland compositor) + wayvnc (VNC server for remote access)
- Sway config with `Mod1` (Alt) modifier to avoid conflict with host Super key
- `/tmp` remounted from tmpfs to disk (prevents OOM crash during Steam extraction)
- `TMPDIR=/var/tmp` for Steam (prevents tmpfs memory pressure during updates)
- `sway-session.service` (Wayland compositor)
- `steam-farm.service` (auto-starts Steam when session files are injected)
- virtiofs mount point at `/opt/cs2`

### 3. Create a single VM
```bash
./target/release/worker create \
  --name cs2-test-1 \
  --ram 2048 \
  --cpus 2 \
  --vnc-port 5901 \
  --base-disk /var/lib/vmctl/base/ubuntu-22.04.qcow2 \
  --disk-dir /var/lib/vmctl/disks \
  --virtiofs-source /opt/cs2-shared
```

### 4. Batch-create multiple VMs
```bash
./target/release/worker setup \
  --base-disk /var/lib/vmctl/base/ubuntu-22.04.qcow2 \
  --count 4 \
  --prefix cs2-farm \
  --ram 2048 \
  --cpus 2 \
  --vnc-start 5901 \
  --disk-dir /var/lib/vmctl/disks \
  --virtiofs-source /opt/cs2-shared
```

### 5. VM lifecycle
```bash
./target/release/worker list
./target/release/worker start cs2-farm-0
./target/release/worker stop cs2-farm-0
./target/release/worker destroy cs2-farm-0    # force-stop
./target/release/worker undefine cs2-farm-0   # remove definition
```

### 6. Connect to VMs

#### Ping guest agent
```bash
./target/release/worker ga-ping cs2-farm-0
```

#### Execute commands inside VM
```bash
# Run any shell command via guest agent (no SSH needed)
./target/release/worker ga-exec cs2-farm-0 --cmd "uname -a"
./target/release/worker ga-exec cs2-farm-0 --cmd "dmidecode -t system"
./target/release/worker ga-exec cs2-farm-0 --cmd "ip link show"
```

#### Connect via VNC (for manual testing)
```bash
# QEMU VNC (framebuffer, always available): localhost:<vnc_port>
vncviewer localhost:5901

# wayvnc (Wayland-native, inside sway session): guest_ip:5900
# wayvnc runs automatically inside the VM via sway config.
# Access it through the guest network or via SSH tunnel:
ssh -L 5900:localhost:5900 farmuser@<vm_ip>
vncviewer localhost:5900
```

### 7. Verify hardware spoofing

After VM is booted and guest agent is running:
```bash
# Verify all spoofed hardware identifiers match inside the VM
./target/release/worker verify cs2-farm-0

# JSON output for automated checks
./target/release/worker verify cs2-farm-0 --json
```

This checks:
- MAC address (should be real vendor OUI, not QEMU default 52:54:00)
- SMBIOS manufacturer, product, serial (via `dmidecode`)
- Disk serial (via `lsblk`)
- Hypervisor visibility (should be hidden from CPUID)
- NIC driver (should be `e1000e`, not `virtio-net`)

### 8. Inspect spoofed identity
```bash
./target/release/worker show-identity cs2-farm-0
```
Output (JSON):
```json
{
  "mac_address": "a4:bb:6d:xx:xx:xx",
  "smbios_manufacturer": "Dell Inc.",
  "smbios_product": "OptiPlex 7080",
  "smbios_serial": "SVC1234567",
  "disk_serial": "SAM12345678",
  "disk_model": "Samsung SSD 870 EVO 500GB"
}
```

### 9. Steam session injection

Inject a Steam account for automatic login (no user interaction needed):
```bash
# Inject session files into a running VM
./target/release/worker inject-session cs2-farm-0 \
  --account mysteamuser \
  --token "eyJhbGciOi..." \
  --steam-id 76561198012345678 \
  --persona "FarmBot"

# Switch to a different account (kills Steam, injects new session, restarts)
./target/release/worker switch-account cs2-farm-0 \
  --account otheruser \
  --token "eyJhbGciOi..." \
  --steam-id 76561198087654321 \
  --persona "FarmBot2"
```

### 10. CS2 updates

CS2 is shared to all VMs via virtiofs from a single host directory:

```bash
# Check current CS2 installation status
./target/release/worker cs2-status --shared-dir /opt/cs2-shared

# Update CS2 (stops CS2 in specified VMs, runs steamcmd, VMs auto-restart)
./target/release/worker cs2-update \
  --shared-dir /opt/cs2-shared \
  --vms cs2-farm-0 --vms cs2-farm-1 --vms cs2-farm-2
```

The update process:
1. Acquires a lock file to prevent concurrent updates
2. Signals VMs to stop CS2 via guest agent
3. Runs `steamcmd +app_update 730 validate`
4. Releases lock
5. VMs auto-restart CS2 via systemd

### 11. virtiofs shared CS2 directory

All VMs share a single CS2 installation from the host:
```bash
# Create shared directory on host
sudo mkdir -p /opt/cs2-shared

# Install CS2 once (or use cs2-update command above)
steamcmd +force_install_dir /opt/cs2-shared +login anonymous +app_update 730 validate +quit

# VMs auto-mount via /etc/fstab (configured by cloud-init):
#   cs2 /opt/cs2 virtiofs defaults,nofail 0 0
```

Benefits:
- **Storage savings**: ~35 GB shared vs ~280 GB for 8 separate copies
- **Instant updates**: Update once on host, all VMs see new files immediately
- **Native performance**: virtiofs has near-native I/O speed

## Architecture

```
vmctl (this binary)
├── check-deps     → validates QEMU, libvirt, KVM, virtiofsd
├── create         → generates spoofed HW identity + qcow2 overlay + libvirt XML
├── setup          → batch-creates N VMs
├── start/stop     → virsh start/shutdown
├── destroy        → virsh destroy (force)
├── undefine       → virsh undefine
├── list           → virsh list --all
├── show-identity  → deterministic HW spoofing preview
├── verify         → check spoofing inside running VM via guest agent
├── ga-ping        → test guest agent connectivity
├── ga-exec        → run shell commands inside VM
├── inject-session → write Steam session files for auto-login
├── switch-account → switch VM to different Steam account
├── cs2-status     → check shared CS2 installation status
├── cs2-update     → coordinate CS2 updates across VMs
└── cloud-init     → generate provisioning ISO for new VMs
```

### Anti-Detection Features (in generated VM XML)

| Feature | Purpose |
|---------|---------|
| `<kvm><hidden state='on'/>` | Hides KVM CPUID leaf |
| `<feature policy='disable' name='hypervisor'/>` | Removes hypervisor flag from CPUID |
| `<hyperv><vendor_id value='GenuineIntel'/>` | Spoofs Hyper-V vendor ID |
| `<vmport state='off'/>` | Disables VMware I/O port |
| `<memballoon model='none'/>` | Removes memory balloon (VM indicator) |
| `<smbios mode='sysinfo'/>` | Activates SMBIOS data injection |
| `<model type='e1000e'/>` | Uses real Intel NIC (not virtio PCI ID 1AF4) |
| Real OUI MAC addresses | Vendor-specific prefixes (Intel, Realtek, Broadcom) |
| Realistic SMBIOS | Dell, HP, Lenovo, ASUS system info with proper serials |

## Next Steps (beyond this PoC)

1. **VNC/QMP controller**: Read VM display buffer, inject keyboard/mouse from the host worker for gameplay automation
2. **shai RPC integration**: Connect to central server for account queue, heartbeat, farm coordination
3. **KSM + memory tuning**: Enable kernel same-page merging for multi-VM density (~30-50% savings)
4. **Venus GPU validation**: Test VirtIO-GPU Venus with CS2 at 384×288 Vulkan rendering
5. **ACPI table patching**: Patch QEMU ACPI tables to remove BOCHS/BXPC strings (hard detection vector)
