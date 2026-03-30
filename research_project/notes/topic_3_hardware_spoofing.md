# Topic 3: VM Hardware Spoofing — MAC, Disk Serial, SMBIOS, CPU/RAM

## 1. MAC Address Randomization

```xml
<interface type='network'>
  <mac address='00:1A:A0:3F:B2:77'/>  <!-- Use real vendor OUI, not 52:54:00 (QEMU) -->
  <source network='default'/>
  <model type='e1000e'/>   <!-- IMPORTANT: use e1000e, NOT virtio-net (exposes VEN_1AF4) -->
</interface>
```

Generate random MAC with Intel OUI:
```bash
printf '00:1A:A0:%02x:%02x:%02x\n' $((RANDOM%256)) $((RANDOM%256)) $((RANDOM%256))
```

## 2. Disk Serial Number Spoofing

```xml
<disk type='file' device='disk'>
  <driver name='qemu' type='qcow2'/>
  <source file='/var/lib/libvirt/images/vm.qcow2'/>
  <target dev='sda' bus='sata'/>
  <serial>WD-WXE1A80K3KJH</serial>   <!-- max 20 chars, look like real drive -->
</disk>
```

Direct QEMU: `-drive file=disk.qcow2,format=qcow2,if=ide,serial=WD-WXE1A80K3KJH`

## 3. SMBIOS Spoofing (libvirt XML)

```xml
<domain type='kvm' xmlns:qemu='http://libvirt.org/schemas/domain/qemu/1.0'>
  <uuid>a3b4c5d6-e7f8-1234-abcd-1234567890ab</uuid>

  <sysinfo type='smbios'>
    <bios>
      <entry name='vendor'>American Megatrends International, LLC.</entry>
      <entry name='version'>F.70</entry>
      <entry name='date'>11/02/2021</entry>
      <entry name='release'>5.16</entry>
    </bios>
    <system>
      <entry name='manufacturer'>ASUS</entry>
      <entry name='product'>ROG STRIX B550-F GAMING</entry>
      <entry name='version'>Rev X.0x</entry>
      <entry name='serial'>M80ABCDEF01234</entry>
      <entry name='uuid'>a3b4c5d6-e7f8-1234-abcd-1234567890ab</entry>
      <entry name='sku'>SKU</entry>
      <entry name='family'>ROG STRIX</entry>
    </system>
    <baseBoard>
      <entry name='manufacturer'>ASUSTeK COMPUTER INC.</entry>
      <entry name='product'>ROG STRIX B550-F GAMING</entry>
      <entry name='serial'>M80ABCDEF01234</entry>
    </baseBoard>
    <chassis>
      <entry name='manufacturer'>Default string</entry>
      <entry name='serial'>Default string</entry>
    </chassis>
  </sysinfo>

  <os>
    <type arch='x86_64' machine='q35'>hvm</type>
    <smbios mode='sysinfo'/>  <!-- REQUIRED to activate sysinfo block -->
  </os>
</domain>
```

## 4. CPU Topology and Hypervisor Hiding

```xml
<cpu mode='host-passthrough' check='none'>
  <topology sockets='1' dies='1' cores='4' threads='2'/>
  <feature policy='disable' name='hypervisor'/>
</cpu>

<features>
  <acpi/>
  <apic/>
  <hyperv mode='custom'>
    <vendor_id state='on' value='GenuineIntel'/>  <!-- Must be 12 chars exactly -->
  </hyperv>
  <kvm>
    <hidden state='on'/>  <!-- Hides KVM CPUID leaf 0x40000000 -->
  </kvm>
  <vmport state='off'/>
</features>
```

CPU brand string (model_id):
```xml
<qemu:commandline>
  <qemu:arg value='-cpu'/>
  <qemu:arg value='host,family=6,model=165,stepping=5,model_id=Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz,hypervisor=off'/>
</qemu:commandline>
```

## 5. RAM and vCPU Variation

```xml
<memory unit='GiB'>16</memory>
<currentMemory unit='GiB'>16</currentMemory>
<vcpu placement='static'>8</vcpu>
<memballoon model='none'/>  <!-- Remove balloon — its presence signals VM -->
```

Use realistic combos: 4/8/16/32 GB RAM with 4/6/8/12 vCPUs (powers of two / common counts).

## 6. Persistent Detection Vectors (Hard to Fix)

| Vector | Mitigation | Difficulty |
|---|---|---|
| RDTSC timing (VM exit cost ~1000-5000 cycles vs ~100 bare metal) | kvm-rdtsc-hack kernel module | Very Hard |
| CPUID KVM leaf | `<kvm><hidden state='on'/></kvm>` | Easy |
| HyperV vendor_id | `<vendor_id value='GenuineIntel'/>` | Easy |
| ACPI table strings (BOCHS/BXPC/QEMU) | qemu-anti-detection patch | Hard |
| Virtio PCI device IDs (VEN_1AF4) | Use e1000e NIC, avoid all virtio | Easy |
| WMI thermal/fan sensors (empty in VMs) | No complete solution | Unsolvable in SW |
| Memory balloon driver presence | `<memballoon model='none'/>` | Easy |
| QEMU Keyboard/Mouse device names | qemu-anti-detection patch | Hard |

## 7. qemu-anti-detection Project

Source-level QEMU patches that:
- Rename `QEMU HARDDISK` → realistic drive names
- Remove `QEMU`/`BOCHS` from ACPI OEM strings
- Rename `QEMU Keyboard`/`QEMU Mouse`

Project: https://github.com/zhaodice/qemu-anti-detection (patches for QEMU 6.2–8.2)

## Sources
- https://libvirt.org/formatdomain.html
- https://github.com/zhaodice/qemu-anti-detection
- https://secret.club/2020/04/13/how-anti-cheats-detect-system-emulation.html
- https://github.com/h33p/kvm-rdtsc-hack
