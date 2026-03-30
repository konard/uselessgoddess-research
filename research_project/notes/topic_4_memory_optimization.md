# Topic 4: Memory Optimization — ZRAM, KSM, Ballooning for Multi-VM Hosts

## 1. ZRAM

Creates a compressed block device in RAM. When used as swap, compressed pages stay in memory instead of going to disk.

**Compression ratios:**
| Algorithm | Ratio | Speed |
|---|---|---|
| lz4 | ~2.6:1 | Fastest |
| lzo-rle | ~2.7–3.7:1 | Fast |
| zstd | ~3.4–5.0:1 | Best ratio, slower compression |

A 4 GB zram device with lz4 might hold ~2.1 GB of actual data using only ~261 MB real RAM.

**Configuration:**
```bash
# Manual setup
modprobe zram
zramctl /dev/zram0 --algorithm zstd --size 4G
mkswap /dev/zram0
swapon /dev/zram0 --priority 100

# systemd-zram-generator (/etc/systemd/zram-generator.conf)
[zram0]
zram-size = min(ram, 8192)
compression-algorithm = zstd
swap-priority = 100
```

**Optimal sysctl for ZRAM-only:**
```ini
vm.swappiness = 150    # Linux 5.8+: >100 strongly prefers zram
vm.vfs_cache_pressure = 500
vm.dirty_background_ratio = 1
vm.dirty_ratio = 50
```

## 2. ZRAM vs Traditional Swap

| Characteristic | ZRAM | Traditional Swap |
|---|---|---|
| Latency | Microseconds (in-RAM) | Milliseconds (HDD) / ~100µs (NVMe) |
| Capacity | Bounded by physical RAM | Bounded by disk space |
| Works without disk | Yes | No |
| Best use case | RAM-constrained, no fast storage | NVMe with unpredictable swap demand |

## 3. zswap vs zram (Critical Distinction)

| Feature | zswap | zram |
|---|---|---|
| What it is | Compressed cache IN FRONT OF disk swap | Standalone compressed device AS swap |
| Requires disk swap | Yes (it's a cache layer) | No |
| LRU correctness | Maintained (shrinker daemon) | Can be inverted (when full → OOM) |
| Default in distros | Yes (Fedora, Ubuntu 22.04+) | Optional |

**Recommendation:** For servers with disk swap, **zswap is preferred** over zram — it has intelligent page eviction and avoids the "full zram → 20-minute hang" problem documented by Cloudflare.

Use ZRAM when: no disk swap available, diskless systems, or you need full control.

## 4. KSM (Kernel Same-page Merging)

KSM scans RAM for identical pages across VMs and deduplicates them (CoW). QEMU/KVM automatically marks guest RAM as MADV_MERGEABLE.

**Savings potential:**
| Scenario | Typical Savings |
|---|---|
| 10 Ubuntu VMs (same OS, idle) | 2–4 GB |
| 50+ identical VMs | 20–40 GB |
| 52 Windows XP VMs (1 GB each) | Run on 16 GB host |
| CS2 game VMs (same binary, same map) | 30–50% deduplication |

**Configuration:**
```bash
echo 1 | sudo tee /sys/kernel/mm/ksm/run
echo 1000 | sudo tee /sys/kernel/mm/ksm/pages_to_scan
echo 50   | sudo tee /sys/kernel/mm/ksm/sleep_millisecs
echo 512  | sudo tee /sys/kernel/mm/ksm/max_page_sharing

# Check savings
pages_sharing=$(cat /sys/kernel/mm/ksm/pages_sharing)
pages_shared=$(cat /sys/kernel/mm/ksm/pages_shared)
echo "KSM saving: $(( (pages_sharing - pages_shared) * 4 / 1024 )) MB"
```

**Note:** KSM is incompatible with huge pages.

## 5. Memory Ballooning (virtio-balloon)

Dynamically reclaims unused VM RAM back to host.

```xml
<memory unit='MiB'>2048</memory>         <!-- Max -->
<currentMemory unit='MiB'>1024</currentMemory>  <!-- Initial -->
<devices>
  <memballoon model='virtio'>
    <stats period='5'/>
  </memballoon>
</devices>
```

```bash
virsh setmem worker-01 2048M --live  # Deflate (give more RAM)
virsh setmem worker-01 512M --live   # Inflate (reclaim RAM)
virsh dommemstat worker-01           # Check stats
```

**Limitations:**
- Not automatic by default (requires scripting or Proxmox)
- Incompatible with huge pages
- Aggressive inflation causes internal guest swapping

**Note for anti-detection:** `<memballoon model='none'/>` removes the balloon device — its presence signals virtualization to detection software.

## 6. CS2 RAM Requirements (Realistic)

- **Official minimum:** 8 GB (client) / 4 GB recommended (dedicated server, 10–12 players)
- **CS2 server binary at idle/warmup:** ~800 MB – 1.2 GB RSS
- **During active match (10 players):** ~1.5–2.0 GB RSS
- **500 MB is NOT viable** for CS2 — the 500 MB figure likely comes from old CS:GO data

With KSM across 8 identical CS2 VMs (same map, same binary):
- Raw per-VM: ~1.5 GB
- After KSM (~40% dedup): ~900 MB effective unique RAM per VM
- 8 VMs × 900 MB + 3 GB host = ~10.2 GB total on host

## 7. Huge Pages for KVM VMs

| Page size | TLB entries for 4 GB VM |
|---|---|
| 4 KB (default) | ~1,000,000 |
| 2 MB (huge pages) | ~2,000 |
| 1 GB | 4 |

**Benefits:** 10–30% improvement for memory-intensive workloads.

**Drawbacks:**
- Memory is pre-reserved, wasted if unused
- **Incompatible with KSM** (huge pages cannot be deduplicated)
- Incompatible with memory ballooning

**Recommendation for multi-VM CS2 farm:** Do NOT use static huge pages. The KSM savings (4–8 GB) exceed TLB benefits. Use THP in `madvise` mode as compromise:
```bash
echo madvise > /sys/kernel/mm/transparent_hugepage/enabled
echo defer+madvise > /sys/kernel/mm/transparent_hugepage/defrag
```

## 8. Strategy for 8+ VMs on Low-Memory Host

| Layer | Technique | Expected Benefit |
|---|---|---|
| 1 | KSM (enable first) | 30–50% shared pages deduplication |
| 2 | Memory ballooning | Dynamic over-provisioning |
| 3 | Host zswap + NVMe swap | Safety net for bursts |
| 4 | THP madvise | Moderate TLB improvement |

**Budget example (32 GB host, 8 CS2 VMs):**
- 8 × 2 GB max provisioned = 16 GB
- KSM saves ~40% of 12 GB active = −4.8 GB
- Host OS + QEMU = 3–4 GB
- **Effective: ~11–13 GB** → 32 GB host runs 8 VMs with headroom

## Sources
- https://chrisdown.name/2026/03/24/zswap-vs-zram-when-to-use-what.html
- https://wiki.archlinux.org/title/Zram
- https://pve.proxmox.com/wiki/Kernel_Samepage_Merging_(KSM)
- https://www.linux-kvm.org/page/KSM
- https://developers.redhat.com/blog/2021/04/27/benchmarking-transparent-versus-1gib-static-huge-page-performance-in-linux-virtual-machines
