# Research Plan: Distributed CS2 Case Farming System

## Main Topic
Design and architecture of a distributed system for farming CS2 cases using KVM/QEMU virtual machines, with VirtIO-GPU Venus, hardware spoofing, memory optimization (zswap), and remote control via custom Rust RPC (shai).

## Research Subtopics / Questions

### Фаза 1: Стартовое исследование (завершено)
- [x] 1. **VM OS Choice**: Linux + OpenBox — подтверждено (Windows 10 исключена)
- [x] 2. **KVM/QEMU + Venus**: VirtIO-GPU Venus — единственный Vulkan без passthrough
- [x] 3. **Hardware Spoofing**: MAC, SMBIOS, CPUID — генерируется программно
- [x] 4. **Memory Optimization**: zswap + NVMe swap + KSM — текущий приоритет
- [x] 5. **CS2 Low-Res Mode**: 384×288, минимальные параметры
- [x] 6. **Input/Display**: VNC для управления, QMP для скриншотов
- [x] 7. **Worker Architecture**: Rust daemon с `virt` crate
- [x] 8. **Parallel VM Management**: qcow2 thin clones, CPU pinning
- [x] 9. **Server-Worker Communication**: shai RPC (QUIC)
- [x] 10. **Security & Evasion**: SMBIOS + CPUID hide достаточно для VAC

### Фаза 2: Анализ кастомного RPC и уточнения
- [x] 11. **Анализ shai RPC**: Архитектура, wire format, extractors, benchmarks
- [x] 12. **Steam Login стратегия**: Предзаполненная сессия (loginusers.vdf + tokens)
- [x] 13. **VM XML управление**: `virt` crate (Rust libvirt bindings) + шаблоны
- [x] 14. **CS2 шаринг между VM**: virtiofs — одна копия CS2 на все VM
- [x] 15. **Стратегия разработки**: Bottom-up (VM → воркер → сервер)

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
- `notes/topic_11_rpc_analysis.md`
- `notes/topic_12_steam_login.md`
- `notes/topic_13_vm_xml_management.md`
- `notes/topic_14_cs2_sharing.md`
- `notes/topic_15_dev_strategy.md`
- `FINAL_REPORT.md`
