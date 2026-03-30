# Topic 10: VM Detection Evasion and Fingerprint Isolation

## 1. Anti-Cheat VM Detection Methods

| Detection Vector | Anti-Cheat Systems | How It Works |
|---|---|---|
| CPUID hypervisor bit | VAC, EAC, BattlEye | CPUID leaf 0x1 ECX bit 31 set = hypervisor present |
| KVM CPUID leaf (0x40000000) | VAC, EAC | Returns "KVMKVMKVM" string |
| RDTSC timing | BattlEye, EAC | VM exits add ~1000–5000 cycles overhead |
| SMBIOS strings (QEMU/BOCHS) | VAC, EAC | `/sys/class/dmi/id/` shows "QEMU" |
| Virtio PCI device IDs | EAC | VEN_1AF4 (Red Hat) in device manager |
| ACPI table signatures | VAC, EAC | BOCHS/BXPC/QEMU in DSDT/SSDT |
| Memory balloon driver | BattlEye | `balloon.sys` / `vioscsi.sys` in drivers list |
| WMI thermal/fan sensors | BattlEye | No fan/temperature data in VMs |
| Disk device name | VAC | "QEMU HARDDISK" string |

## 2. Mitigation Techniques

### Hide KVM/Hypervisor from Guest

```xml
<features>
  <acpi/>
  <apic/>
  <hyperv mode='custom'>
    <vendor_id state='on' value='GenuineIntel'/>
  </hyperv>
  <kvm>
    <hidden state='on'/>  <!-- Hides KVM CPUID leaf -->
  </kvm>
  <vmport state='off'/>
</features>

<cpu mode='host-passthrough' check='none'>
  <feature policy='disable' name='hypervisor'/>
</cpu>
```

### CPUID / CPU Spoofing

```xml
<qemu:commandline>
  <qemu:arg value='-cpu'/>
  <qemu:arg value='host,hypervisor=off,vmware-cpuid-freq=false,model_id=Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz'/>
</qemu:commandline>
```

### Remove Virtio Devices (Use Emulated Hardware Instead)

```xml
<!-- NIC: e1000e instead of virtio-net -->
<interface type='network'>
  <model type='e1000e'/>
</interface>

<!-- No virtio balloon -->
<memballoon model='none'/>

<!-- No virtio serial channels -->
<!-- Remove: <channel type='spicevmc'> -->
<!-- Remove: <channel type='unix' target name='org.qemu.guest_agent.0'> -->
```

### ACPI Table Patching (Advanced)

The `qemu-anti-detection` project patches QEMU source to:
- Replace `BOCHS`/`BXPC`/`QEMU` strings in ACPI tables
- Rename `QEMU HARDDISK` → realistic drive model names
- Rename `QEMU DVD-ROM` → real drive models
- Rename `QEMU Keyboard`/`QEMU Mouse` → brand names

https://github.com/zhaodice/qemu-anti-detection

### RDTSC Timing (Hardest to Fix)

The RDTSC timing attack measures the overhead of VM exits:
- Bare metal: ~100 CPU cycles for `RDTSC; CPUID; RDTSC`
- KVM: ~1000–5000+ cycles

Mitigation: `kvm-rdtsc-hack` kernel module adjusts the vRDTSC offset to hide exit overhead.
https://github.com/h33p/kvm-rdtsc-hack

## 3. Detection Effectiveness by Anti-Cheat

| Anti-Cheat | Basic Spoof (SMBIOS/CPUID) | + qemu-anti-detection | + RDTSC fix |
|---|---|---|---|
| **VAC (Steam)** | High | Very High | Complete |
| **EAC (Easy Anti-Cheat)** | Medium | High | High |
| **BattlEye** | Low-Medium | Medium | High |
| **Vanguard (Riot)** | Low | Low-Medium | Medium |

**VAC is the relevant anti-cheat for CS2.** It is server-side and does not use kernel-level detection. The SMBIOS + CPU spoofing in Topic 3 provides high effectiveness against VAC.

## 4. Network-Level Isolation

For maximum account isolation, each VM should appear to come from a different network:

```
Option A: Each VM gets its own IP via ISP
  → Host has multiple IP addresses, each NAT'd to one VM

Option B: Residential proxies per VM
  → VM traffic routed through different residential IP proxies

Option C: VPN per VM
  → Each VM runs its own VPN connection (e.g., WireGuard)
  → WireGuard config per VM
```

**WireGuard per VM example:**
```bash
# In VM: bring up WireGuard with dedicated VPN account
wg-quick up wg0  # where wg0 uses a unique peer/IP
```

If all VMs share one IP, account bans may be correlated.

## 5. Behavioral Fingerprinting

Even with hardware spoofing, behavioral patterns can correlate accounts:
- Login/logout timing patterns
- Gaming sessions always same length
- Same map/game mode rotations
- Mouse/keyboard movement patterns (robotic vs human)

Mitigations:
- Randomize session lengths ±20%
- Randomize start times (human-like schedule)
- Add jitter to mouse movements
- Use different maps/modes per account

## 6. Practical Checklist for VM Anti-Detection

- [ ] `<kvm><hidden state='on'/></kvm>` — hides KVM leaf
- [ ] `<vendor_id state='on' value='GenuineIntel'/>` — spoof HyperV vendor
- [ ] `<feature policy='disable' name='hypervisor'/>` — disable hypervisor CPUID bit
- [ ] SMBIOS type 0,1,2,3 spoofed with real hardware values
- [ ] NIC model `e1000e` with real vendor OUI MAC
- [ ] Disk serial matching realistic drive brand
- [ ] `<memballoon model='none'/>` — no balloon driver
- [ ] CPU model_id matches real CPU brand string
- [ ] No virtio serial/spice channels
- [ ] Per-VM unique UUID, serial, MAC
- [ ] (Advanced) ACPI table patching via qemu-anti-detection
- [ ] (Advanced) RDTSC fix via kvm-rdtsc-hack kernel module
- [ ] Per-VM network isolation (separate IPs or proxies)

## Sources
- https://secret.club/2020/04/13/how-anti-cheats-detect-system-emulation.html
- https://github.com/zhaodice/qemu-anti-detection
- https://github.com/h33p/kvm-rdtsc-hack
- https://arxiv.org/html/2502.12322v1 (VIC: Evasive Video Game Cheating via VMI)
