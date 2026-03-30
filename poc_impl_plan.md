# PoC Implementation Plan — VM Worker CLI

## Architecture Overview

The worker CLI (`vmctl`) manages KVM/QEMU virtual machines for CS2 case farming.
It runs on any Linux host with KVM support and manages VM lifecycle, hardware spoofing,
guest connectivity, session injection, and CS2 update coordination.

### Module Breakdown

- [x] **1. Dependency Checker** (`deps.rs`) — Validate host: QEMU version ≥ 9.2, libvirt, KVM module, virtiofs, qemu-img, required kernel modules.
- [x] **2. Hardware Spoofing** (`spoof.rs`) — Generate realistic MAC (real OUI), SMBIOS (manufacturer/product/serial), disk serials. Deterministic per-VM seed.
- [x] **3. VM Config / XML Generation** (`config.rs`) — Produce libvirt-compatible XML from structured Rust types. CPU/RAM, VirtIO-GPU Venus, spoofed identifiers, virtiofs mounts, VNC display. Enhanced anti-detection: KVM hidden, Hyper-V vendor_id, vmport off, memballoon disabled, smbios sysinfo mode, memory backing for virtiofs.
- [x] **4. Disk Management** (`disk.rs`) — qcow2 backing store + per-VM overlay creation via `qemu-img`.
- [x] **5. VM Lifecycle** (`vm.rs`) — Create, start, stop, destroy, list VMs. Wraps `virsh` commands (no libvirt C bindings needed for PoC).
- [x] **6. CLI Interface** (`main.rs`) — Subcommands: `check-deps`, `create`, `start`, `stop`, `destroy`, `list`, `setup`, `verify`, `ga-ping`, `ga-exec`, `inject-session`, `switch-account`, `cs2-status`, `cs2-update`, `cloud-init`.
- [x] **7. Guest Agent** (`guest_agent.rs`) — QEMU Guest Agent interface via `virsh qemu-agent-command`. Ping, exec, file read/write, network queries. No network required — uses virtio-serial channel.
- [x] **8. Spoofing Verification** (`verify.rs`) — Verify hardware spoofing inside running VMs via guest agent. Checks MAC, SMBIOS, disk serial, hypervisor visibility, NIC driver.
- [x] **9. Base Image Setup** (`image.rs`) — Cloud-init configuration generation for automatic VM provisioning. Creates user-data/meta-data with qemu-guest-agent, Steam, mesa/Vulkan, OpenBox, systemd services.
- [x] **10. CS2 Update Management** (`update.rs`) — Centralized CS2 update strategy using virtiofs shared directory. Lock-based coordination, steamcmd integration, VM notification.
- [x] **11. Steam Session Injection** (`session.rs`) — Inject Steam session files (config.vdf, loginusers.vdf) via guest agent for automatic login. Refresh token based, no user interaction needed. Account switching support.
- [x] **12. Unit Tests** — 60 tests covering: spoofing determinism, XML generation, anti-detection features, base64 encoding, VDF generation, cloud-init, update lock lifecycle, verification parsing.

### Non-goals for this PoC
- No RPC/server integration (Phase 3)
- No VNC framebuffer automation / AI inference (Phase 4)
- No actual steamcmd execution (requires Steam account)

### Design Decisions
- **No libvirt C bindings**: Use `virsh` CLI wrapper for PoC simplicity and portability.
- **Hybrid approach (no mini-worker)**: Uses QEMU Guest Agent (standard component) + systemd service inside VM, controlled from host. No custom daemon inside guest reduces VAC detection risk.
- **Deterministic spoofing**: Each VM gets reproducible hardware IDs from a seed (VM name hash).
- **OS-agnostic**: No hard dependency on specific distro; checks required tools at runtime.
- **virtiofs for CS2 sharing**: Single CS2 installation on host shared to all VMs via virtiofs (saves ~30GB per VM).
- **Refresh token injection**: Steam auto-login via refresh tokens written to config.vdf (valid ~200 days).
- **cloud-init provisioning**: Automated VM setup with all required packages and services.
- **Anti-detection hardening**: KVM hidden, hypervisor CPUID disabled, Hyper-V vendor_id spoofed, vmport off, memballoon disabled, e1000e NIC (not virtio-net).
