# PoC Implementation Plan — Container Worker CLI

## Architecture Overview

The worker CLI manages Docker containers for CS2 case farming.
It runs on any Linux host with AMD/Intel GPU support.

```
Host (Docker + AMD/Intel GPU)
├── worker binary
│   ├── create       → docker run + GPU + spoof mounts + VNC
│   ├── exec         → docker exec (command execution)
│   └── verify       → check spoofing via docker exec
├── /var/lib/vmctl/container-spoof/{name}/dmi/  (fake DMI files)
├── /opt/cs2-shared/  (bind-mounted read-only into containers)
└── Containers: Ubuntu 24.04 + Sway + wayvnc + /dev/dri
```

## Module Breakdown

- [x] **1. Container Dockerfile** (`container/Dockerfile`) — Ubuntu 24.04 base with Mesa/Vulkan (AMD radv, Intel anv), Sway, wayvnc, XWayland, Steam dependencies. GPU via /dev/dri passthrough.
- [x] **2. Container Entrypoint** (`container/entrypoint.sh`) — Sets up XDG runtime, applies spoofed machine-id, starts Sway compositor and steam-launcher.
- [x] **3. Hardware Spoofing** (`container/spoof.rs`) — Generate fake DMI files (/sys/class/dmi/id/), machine-id, and Docker bind-mount arguments. Deterministic per-container. MAC spoofing via Docker --mac-address flag.
- [x] **4. Identity Generation** (`spoof.rs`) — Generate realistic MAC (real OUI), SMBIOS (manufacturer/product/serial), disk serials. Deterministic from container name hash.
- [x] **5. Container Exec** (`container/exec.rs`) — Docker exec wrapper. Command execution, file read/write via docker exec + tee/cat, docker cp for large files.
- [x] **6. Container Lifecycle** (`container/mod.rs`) — Create, start, stop, kill, remove, list containers via Docker CLI. GPU passthrough (/dev/dri), MAC/DMI spoofing, CS2 shared mount, VNC port mapping.
- [x] **7. Steam Session Injection** (`container/session.rs`) — Steam session injection via docker exec. Writes config.vdf + loginusers.vdf with refresh token for auto-login.
- [x] **8. Display Management** (`container/display.rs`) — Wayland display management. Check display readiness (Sway + wayvnc), screenshot via grim, swaymsg input.
- [x] **9. Spoofing Verification** (`container/verify.rs`) — Verify spoofing inside containers. Checks DMI files, machine-id, MAC address, GPU render node, Vulkan availability.
- [x] **10. Dependency Checks** (`container/deps.rs`) — Check Docker daemon, GPU render nodes, Docker image, CS2 shared directory.
- [x] **11. CS2 Updates** (`container/update.rs`) — Lock-based update strategy. Notifies containers via docker exec to stop CS2, runs steamcmd on host, containers auto-restart.
- [x] **12. CLI** (`main.rs`) — Subcommands: create, setup, start, stop, destroy, list, exec, verify, show-identity, inject-session, switch-account, cs2-status, cs2-update, display-status, screenshot, check-deps.
- [x] **13. Unit Tests** — 38 tests covering: spoofing determinism, VDF generation, Docker arg construction, state serialization, VNC port extraction, update lock lifecycle, verification.

## Design Decisions

- **Deterministic spoofing**: Each container gets reproducible hardware IDs from a name hash.
- **Shared CS2**: Single CS2 installation on host bind-mounted read-only into all containers.
- **Refresh token injection**: Steam auto-login via refresh tokens written to config.vdf (valid ~200 days).
- **Container spoofing**: Bind-mount fake DMI files over /sys/class/dmi/id/, custom machine-id, Docker MAC address flag.
- **Wayland + Sway**: Headless Wayland compositor with wayvnc for VNC access. XWayland for Steam/CS2 compatibility.
- **GPU passthrough via /dev/dri**: AMD/Intel render nodes shared across containers (native performance, no VM overhead).
- **No hypervisor artifacts**: Containers share host kernel — no CPUID hypervisor bit, no KVM-related detections.

## Non-goals for this PoC

- No RPC/server integration (Phase 3)
- No VNC framebuffer automation / AI inference (Phase 4)
- No actual steamcmd execution (requires Steam account)
