# Topic 1: VM OS Choice — Windows 10 LTSC vs Linux + OpenBox + CS2 Native (Vulkan)

## 1. CS2 Native Linux Support

CS2 has a **first-party native Linux build** shipped by Valve. It uses Vulkan as its primary rendering API.
- Official minimum GPU requirement: AMD GCN+ or NVIDIA Kepler+ with up-to-date Vulkan drivers
- Extension `VK_EXT_graphics_pipeline_library` highly recommended
- No OpenGL or DirectX requirement for Linux — Vulkan is mandatory
- ProtonDB and GamingOnLinux confirm "Native" Linux status as of 2026

## 2. Resource Usage Comparison

| OS Configuration | Typical Idle RAM |
|---|---|
| Windows 10 LTSC (IoT Enterprise) | 900 MB – 1.5 GB |
| Windows 10 Home/Pro | 1.5 GB – 2.5 GB |
| Ubuntu 22.04 LTS (GNOME) | ~750 MB – 1 GB |
| Debian/Arch + OpenBox + X11 | ~150 MB – 350 MB |
| Debian/Arch + i3wm + X11 | ~100 MB – 250 MB |

CPU idle: minimal Linux (~0%), Windows LTSC (~20–50 background processes).
Boot time: Linux + OpenBox: 5–15s cold; Windows 10 LTSC: 20–40s cold.

## 3. Anti-Cheat (VAC) Compatibility

VAC works fully on Linux — both native and via Proton. CS2 does NOT use kernel-level anti-cheat.
- GamingOnLinux Anti-Cheat Tracker: CS2 status = "Works"
- Source: https://www.gamingonlinux.com/anticheat/?search=Counter-Strike+2

## 4. GPU/Driver Requirements in VM Context

Critical caveat: CS2 requires Vulkan.

| Option | Viable for CS2 |
|---|---|
| QXL (2D only) | NO |
| VirtIO-GPU virgl (OpenGL only) | NO (CS2 needs Vulkan) |
| VirtIO-GPU Venus (Vulkan) | YES (QEMU 9.2+, kernel 6.13+, Mesa 24.2+) |
| GPU Passthrough (VFIO) | YES (best performance) |
| Software rendering (lavapipe) | Technically possible, <15 FPS |

## 5. Minimum RAM

Official Steam minimum for both platforms: **8 GB RAM**
- Windows LTSC overhead: ~1–1.5 GB leaving ~6.5–7 GB for CS2
- Linux + OpenBox overhead: ~200–350 MB leaving ~7.6–7.8 GB for CS2

## 6. Comparison Table

| Factor | Windows 10 LTSC | Linux + OpenBox |
|---|---|---|
| CS2 native support | Yes (DirectX 11) | Yes (Vulkan, first-party) |
| VAC compatibility | Full | Full (Valve-supported) |
| Idle RAM usage | ~1–1.5 GB | ~150–350 MB |
| Boot time (cold, SSD) | 20–40 seconds | 5–15 seconds |
| VM GPU driver for CS2 | Passthrough required (Venus doesn't support Windows guest) | Venus (Vulkan) viable |
| Vulkan support in VM | Via Proton/DXVK only | Native via Venus layer |
| Storage requirement | 85 GB | 85 GB |
| NVIDIA Error 43 in VM | Yes (needs hypervisor spoof) | No such issue |

## Conclusion

**Linux + OpenBox wins** for VM-based CS2 farming:
- 5–8x less idle RAM
- Faster boot
- Native Vulkan CS2 build with working VAC
- Venus Vulkan layer works on Linux guest (not Windows guest)
- No NVIDIA Error 43 problem

## Sources
- Steam Store: https://store.steampowered.com/app/730/CounterStrike_2/
- ProtonDB: https://www.protondb.com/app/730
- GamingOnLinux: https://www.gamingonlinux.com/anticheat/?search=Counter-Strike+2
- QEMU Venus guide: https://gist.github.com/peppergrayxyz/fdc9042760273d137dddd3e97034385f
