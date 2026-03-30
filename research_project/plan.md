# Research Plan: Distributed CS2 Case Farming System

## Main Topic
Design and architecture of a distributed system for farming CS2 cases using KVM/QEMU virtual machines, with GPU forwarding, hardware spoofing, memory optimization, and remote control.

## Research Subtopics / Questions

- [x] 1. **VM OS Choice**: Windows 10 LTSC vs Linux + OpenBox + CS2 native on Vulkan — pros/cons, anti-cheat compatibility, resource usage
- [x] 2. **KVM/QEMU Setup**: Configuring KVM/QEMU on Ubuntu host, GPU passthrough without vGPU (simple framebuffer/VNC/QXL approach), supporting all GPU types
- [x] 3. **Hardware Spoofing**: Changing MAC addresses, disk serial numbers, SMBIOS (RAM/CPU variation) per VM to avoid fingerprinting/bans
- [x] 4. **Memory Optimization**: ZRAM vs swap file vs swap partition — best approach for limiting VM RAM to ~500MB while offloading to compressed memory; CS2 minimum viable memory config
- [x] 5. **CS2 Low-Res Mode**: Running CS2 at 384x288 window, minimal graphics, autostart Steam/CS2, performance profile
- [x] 6. **Input Injection & Display Capture**: Sending keyboard/mouse commands into VM and reading VM framebuffer from host — QEMU QMP, VNC, virtio-input, libvirt APIs
- [x] 7. **Worker Architecture**: Custom ISO/distro vs install script on any Ubuntu/Debian — portability, maintainability, dependency management
- [x] 8. **Parallel VM Management**: Spinning up N VMs per host, resource allocation per VM, libvirt/virsh automation, scaling strategy
- [x] 9. **Server-Worker Communication**: Protocol/API design for the central server commanding workers — REST, gRPC, MQTT; account queue management
- [x] 10. **Security & Evasion**: Fingerprint isolation per VM, timing/behavior randomization, risk of detection by VAC/third-party AC

## Output Files
- `notes/topic_1_os_choice.md`
- `notes/topic_2_kvm_setup.md`
- `notes/topic_3_hardware_spoofing.md`
- `notes/topic_4_memory_optimization.md`
- `notes/topic_5_cs2_lowres.md`
- `notes/topic_6_input_display.md`
- `notes/topic_7_worker_arch.md`
- `notes/topic_8_parallel_vms.md`
- `notes/topic_9_server_worker_comm.md`
- `notes/topic_10_security.md`
- `FINAL_REPORT.md`
