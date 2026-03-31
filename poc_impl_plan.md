# PoC Implementation Plan — VM + Container Worker CLI

## Architecture Overview

The worker CLI (`vmctl`) manages both KVM/QEMU virtual machines and Docker containers
for CS2 case farming. It runs on any Linux host and provides two parallel backends:

- **VM mode**: Full KVM/QEMU VMs with VirtIO-GPU Venus, libvirt management, SMBIOS spoofing.
- **Container mode**: Lightweight Docker containers with GPU passthrough via /dev/dri,
  file-based hardware spoofing, and Wayland (Sway) + wayvnc for display.

## Phase 1: VM Worker (completed)

### Module Breakdown

- [x] **1. Dependency Checker** (`deps.rs`) — Validate host: QEMU version ≥ 9.2, libvirt, KVM module, virtiofs, qemu-img, required kernel modules.
- [x] **2. Hardware Spoofing** (`spoof.rs`) — Generate realistic MAC (real OUI), SMBIOS (manufacturer/product/serial), disk serials. Deterministic per-VM seed.
- [x] **3. VM Config / XML Generation** (`config.rs`) — Produce libvirt-compatible XML from structured Rust types. CPU/RAM, VirtIO-GPU Venus, spoofed identifiers, virtiofs mounts, VNC display. Enhanced anti-detection: KVM hidden, Hyper-V vendor_id, vmport off, memballoon disabled, smbios sysinfo mode, memory backing for virtiofs.
- [x] **4. Disk Management** (`disk.rs`) — qcow2 backing store + per-VM overlay creation via `qemu-img`.
- [x] **5. VM Lifecycle** (`vm.rs`) — Create, start, stop, destroy, list VMs. Wraps `virsh` commands (no libvirt C bindings needed for PoC).
- [x] **6. CLI Interface** (`main.rs`) — All VM and container subcommands.
- [x] **7. Guest Agent** (`guest_agent.rs`) — QEMU Guest Agent interface via `virsh qemu-agent-command`. Ping, exec, file read/write, network queries. No network required — uses virtio-serial channel.
- [x] **8. Spoofing Verification** (`verify.rs`) — Verify hardware spoofing inside running VMs via guest agent. Checks MAC, SMBIOS, disk serial, hypervisor visibility, NIC driver.
- [x] **9. Base Image Setup** (`image.rs`) — Cloud-init configuration generation for automatic VM provisioning. Creates user-data/meta-data with qemu-guest-agent, Steam (silent install via debconf), mesa/Vulkan, sway + wayvnc, systemd services. Fixes: /tmp remount from tmpfs to disk, TMPDIR=/var/tmp for Steam, sway with Mod1 (Alt) modifier.
- [x] **10. CS2 Update Management** (`update.rs`) — Centralized CS2 update strategy using virtiofs shared directory. Lock-based coordination, steamcmd integration, VM notification.
- [x] **11. Steam Session Injection** (`session.rs`) — Inject Steam session files (config.vdf, loginusers.vdf) via guest agent for automatic login. Refresh token based, no user interaction needed. Account switching support.
- [x] **12. Unit Tests** — 60 tests covering: spoofing determinism, XML generation, anti-detection features, base64 encoding, VDF generation, cloud-init, update lock lifecycle, verification parsing.

## Phase 2: Container Worker (completed)

### Module Breakdown

- [x] **13. Container Dockerfile** (`container/Dockerfile`) — Ubuntu 24.04 base with Mesa/Vulkan (AMD radv, Intel anv), Sway, wayvnc, XWayland, Steam 32/64-bit dependencies. Multi-layer build for efficient caching. GPU via /dev/dri passthrough.
- [x] **14. Container Entrypoint** (`container/entrypoint.sh`) — Sets up XDG runtime, applies spoofed machine-id, starts Sway compositor and steam-launcher.
- [x] **15. Container Hardware Spoofing** (`container_spoof.rs`) — Generate fake DMI files (/sys/class/dmi/id/), machine-id, and Docker bind-mount arguments. Deterministic per-container. MAC spoofing via Docker --mac-address flag.
- [x] **16. Container Exec** (`container_exec.rs`) — Docker exec wrapper replacing QEMU Guest Agent. Command execution, file read/write via docker exec + tee/cat, docker cp for large files.
- [x] **17. Container Lifecycle** (`container.rs`) — Create, start, stop, kill, remove, list containers via Docker CLI. GPU passthrough (/dev/dri), MAC/DMI spoofing, CS2 shared mount, VNC port mapping.
- [x] **18. Container Session Injection** (`container_session.rs`) — Steam session injection via docker exec. Same VDF generation as VM mode but writes files via docker exec instead of guest agent.
- [x] **19. Container Display** (`container_display.rs`) — Wayland display management. Check display readiness (Sway + wayvnc), screenshot via grim, VNC address helper, swaymsg input.
- [x] **20. Container Verification** (`container_verify.rs`) — Verify spoofing inside containers. Checks DMI files, machine-id, MAC address, GPU render node, Vulkan availability.
- [x] **21. Container Dependencies** (`container_deps.rs`) — Check Docker daemon, GPU render nodes, Docker image, CS2 shared directory.
- [x] **22. Container CS2 Updates** (`container_update.rs`) — Same lock-based update strategy as VMs. Notifies containers via docker exec instead of guest agent.
- [x] **23. Container CLI Commands** (`main.rs`) — 15 new container-* subcommands mirroring VM commands.
- [x] **24. Unit Tests** — 88 total tests (28 new container tests) covering: spoof file generation, machine-id determinism, Docker arg construction, state serialization, VNC port extraction, container lifecycle.

### Non-goals for this PoC
- No RPC/server integration (Phase 3)
- No VNC framebuffer automation / AI inference (Phase 4)
- No actual steamcmd execution (requires Steam account)

### Design Decisions
- **No libvirt C bindings**: Use `virsh` CLI wrapper for PoC simplicity and portability.
- **Hybrid approach (no mini-worker)**: Uses QEMU Guest Agent (VM) or docker exec (container) + systemd/scripts inside guest. No custom daemon reduces VAC detection risk.
- **Deterministic spoofing**: Each VM/container gets reproducible hardware IDs from a seed (name hash).
- **OS-agnostic**: No hard dependency on specific distro; checks required tools at runtime.
- **Shared CS2**: Single CS2 installation on host shared to all VMs (virtiofs) / containers (bind mount).
- **Refresh token injection**: Steam auto-login via refresh tokens written to config.vdf (valid ~200 days).
- **Container spoofing**: Bind-mount fake DMI files over /sys/class/dmi/id/, custom machine-id, Docker MAC address flag.
- **Wayland + Sway**: Headless Wayland compositor with wayvnc for display access. XWayland for Steam/CS2 compatibility.
- **GPU passthrough via /dev/dri**: AMD/Intel render nodes shared across containers (no passthrough overhead like VMs).
- **Anti-detection (VM)**: KVM hidden, hypervisor CPUID disabled, Hyper-V vendor_id spoofed, vmport off, memballoon disabled, e1000e NIC.
- **Anti-detection (container)**: DMI spoofing, unique machine-id, MAC randomization, no hypervisor artifacts (containers share host kernel).
