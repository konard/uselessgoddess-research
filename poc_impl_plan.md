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
- [x] **14. Steam Authentication** (`container/steam_auth.rs`) — Automated Steam login via Web API: RSA password encryption, BeginAuthSessionViaCredentials, Steam Guard TOTP from shared_secret (HMAC-SHA1, Steam charset), PollAuthSessionStatus to obtain refresh token.
- [x] **15. Steam Library Configuration** (`container/steam_library.rs`) — Inject libraryfolders.vdf to register /opt/cs2 as a Steam library folder. Prevents CS2 re-download by recognizing shared install.
- [x] **16. Auto-Start Flow** (`main.rs`) — Full automation: steam-login → inject-library → inject-session → Steam+CS2 auto-launch. New CLI commands: steam-login, steam-guard-code, auto-start, inject-library.
- [x] **17. Steam Launcher Update** (`container/steam-launcher.sh`) — Updated to configure /opt/cs2 as library folder fallback before launching Steam+CS2.
- [x] **18. Additional Tests** — 49 total tests (+11 new): TOTP generation, charset validation, credential serialization, library VDF generation.

## Design Decisions

- **Deterministic spoofing**: Each container gets reproducible hardware IDs from a name hash.
- **Shared CS2**: Single CS2 installation on host bind-mounted read-only into all containers.
- **Refresh token injection**: Steam auto-login via refresh tokens written to config.vdf (valid ~200 days).
- **Container spoofing**: Bind-mount fake DMI files over /sys/class/dmi/id/, custom machine-id, Docker MAC address flag.
- **Wayland + Sway**: Headless Wayland compositor with wayvnc for VNC access. XWayland for Steam/CS2 compatibility.
- **GPU passthrough via /dev/dri**: AMD/Intel render nodes shared across containers (native performance, no VM overhead).
- **No hypervisor artifacts**: Containers share host kernel — no CPUID hypervisor bit, no KVM-related detections.
- **Steam Web API login**: Uses IAuthenticationService endpoints (GetPasswordRSAPublicKey, BeginAuthSessionViaCredentials, UpdateAuthSessionWithSteamGuardCode, PollAuthSessionStatus) for automated credential-based login.
- **TOTP from shared_secret**: Generates 5-char Steam Guard codes using HMAC-SHA1 with Steam's custom character set (23456789BCDFGHJKMNPQRTVWXY), matching node-steam-totp algorithm.
- **Library folder injection**: Registers /opt/cs2 mount as Steam library via libraryfolders.vdf, so Steam recognizes CS2 without re-downloading.

## Non-goals for this PoC

- No RPC/server integration (Phase 3)
- No VNC framebuffer automation / AI inference (Phase 4)
