# Topic 2: KVM/QEMU Setup — GPU Access Without Passthrough or vGPU

## Overview of Options

| Option | 3D Accel | Vulkan | Windows Guest | Gaming Viability |
|---|---|---|---|---|
| VirtIO-GPU (virgl) | Yes (OpenGL 4.3) | No | Limited | Poor |
| VirtIO-GPU Venus | Yes (Vulkan 1.3) | Yes | Linux only | Moderate |
| QXL | 2D only | No | Yes | Very poor |
| llvmpipe (CPU software) | Yes | Yes (lavapipe) | N/A | Extremely poor |
| Looking Glass (KVMFR) | Full GPU | Yes | Yes | Excellent (PASSTHROUGH ONLY) |

## VirtIO-GPU virgl (VirGL)

- OpenGL 4.3 + GLES 3.2 support
- **No Vulkan** — CS2 on Linux requires Vulkan, so virgl CANNOT run CS2
- Performance: ~20% of native (677 vs 3391 in vkmark)
- QEMU command: `-device virtio-vga-gl -display sdl,gl=on`

## VirtIO-GPU Venus (Vulkan 1.3) — The Key Option

Requirements:
- **Mesa ≥ 24.2** (August 2024)
- **Linux kernel ≥ 6.13** (late 2024)
- **QEMU ≥ 9.2.0** (November 2024)
- Linux guest ONLY (no Windows guest support yet)
- Requires host GPU with Vulkan support
- Not available on pre-GFX9 AMD (pre-RX 400 series)

QEMU command:
```bash
-device virtio-vga-gl,blob=on,hostmem=4G,venus=on \
-vga none \
-display sdl,gl=on \
-object memory-backend-memfd,id=mem1,size=4G \
-machine memory-backend=mem1
```

Performance: ~43% of native in vkmark (1456 vs 3391 native).

## QXL Display

- 2D acceleration only — **useless for CS2**
- Use only for SPICE remote desktop access, not gaming

## Host GPU Brand Compatibility (Universal Approach)

| Approach | NVIDIA | AMD | Intel iGPU |
|---|---|---|---|
| VirGL (OpenGL) | Yes | Yes | Yes |
| Venus (Vulkan) | Yes* | Yes (GFX9+) | Yes (Gen12+ recommended) |
| QXL / VNC | Yes | Yes | Yes |
| llvmpipe (CPU) | Yes | Yes | Yes |

*NVIDIA may need Mesa workarounds

## Framebuffer Capture Methods (Host reads VM display)

| Method | How | Format | Notes |
|---|---|---|---|
| QEMU QMP `screendump` | JSON over Unix socket | PPM | Works headless |
| libvirt `virDomainScreenshot()` | Python API | PPM stream | Hypervisor-agnostic |
| VNC client library | TCP/Unix to QEMU VNC | Raw RGBA | Continuous stream |
| SPICE client | spice-protocol | Compressed video | Higher-level |
| Looking Glass / KVMFR | Shared memory (IVSHMEM) | Raw RGBA | **REQUIRES PASSTHROUGH** |

## Key Findings for CS2 Without Passthrough

1. **Venus is the only viable non-passthrough Vulkan path** for CS2 on Linux
2. At 384x288, Venus performance overhead (~57%) may be tolerable
3. **Looking Glass requires GPU passthrough** — not applicable without second GPU
4. **QXL provides no 3D** — modern games cannot use it
5. For screen capture: QEMU QMP `screendump` or VNC client libraries work with any display backend

## Recommended Architecture (No Passthrough)

```
Host: Linux with Vulkan-capable GPU (QEMU 9.2+, kernel 6.13+, Mesa 24.2+)

VM:
  -device virtio-vga-gl,blob=on,hostmem=4G,venus=on
  -display vnc=:1  (for framebuffer capture)

Guest: Ubuntu 24.04+ (kernel 6.13+, Mesa 24.2+)
CS2: -w 384 -h 288 -windowed -vulkan

Host capture: VNC client library or QEMU QMP screendump
```

## Sources
- https://www.collabora.com/news-and-blog/blog/2025/01/15/the-state-of-gfx-virtualization-using-virglrenderer/
- https://gist.github.com/peppergrayxyz/fdc9042760273d137dddd3e97034385f
- https://docs.mesa3d.org/drivers/venus.html
- https://wiki.archlinux.org/title/QEMU/Guest_graphics_acceleration
- https://looking-glass.io/docs/B7/requirements/
