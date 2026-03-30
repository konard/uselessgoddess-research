Title: zswap - ArchWiki

URL Source: https://wiki.archlinux.org/title/Zswap

Published Time: Mon, 30 Mar 2026 01:02:22 GMT

Markdown Content:
[zswap](https://en.wikipedia.org/wiki/zswap "wikipedia:zswap") is a kernel feature that provides a compressed RAM cache for swap pages. Pages which would otherwise be swapped out to disk are instead compressed and stored into a memory pool in RAM. Once the pool is full or the RAM is exhausted, the least recently used ([LRU](https://en.wikipedia.org/wiki/Cache_replacement_policies#Least_recently_used_.28LRU.29 "wikipedia:Cache replacement policies")) page is decompressed and written to disk, as if it had not been intercepted. After the page has been decompressed into the swap cache, the compressed version in the pool can be freed.

The difference compared to [zram](https://wiki.archlinux.org/title/Zram "Zram") is that _zswap_ works in conjunction with a [swap](https://wiki.archlinux.org/title/Swap "Swap") device while _zram_ with swap created on top of it is a swap device in RAM that does not require a backing swap device.

## Toggling zswap

**Tip**[Officially supported kernels](https://wiki.archlinux.org/title/Kernel#Officially_supported_kernels "Kernel") have _zswap_ enabled by default. This can be verified with `zgrep CONFIG_ZSWAP_DEFAULT_ON /proc/config.gz`.

_zswap_ can be toggled at runtime, by writing either `1` (to enable) or `0` (to disable) to `/sys/module/zswap/parameters/enabled`. For example, to disable it at runtime:

# echo 0 > /sys/module/zswap/parameters/enabled

To disable _zswap_ permanently on kernels where it is enabled by default, add `zswap.enabled=0` to your [kernel parameters](https://wiki.archlinux.org/title/Kernel_parameters "Kernel parameters").

## Customizing zswap

### Current parameters

_zswap_ has several customizable parameters. The live settings can be displayed using:

$ grep -r . /sys/module/zswap/parameters//sys/module/zswap/parameters/enabled:Y
/sys/module/zswap/parameters/shrinker_enabled:Y
/sys/module/zswap/parameters/max_pool_percent:20
/sys/module/zswap/parameters/compressor:zstd
/sys/module/zswap/parameters/accept_threshold_percent:90

See the [zswap documentation](https://docs.kernel.org/admin-guide/mm/zswap.html) for the description of the different parameters.

The boot time load message showing the initial configuration can be retrieved with:

# dmesg | grep zswap:[    0.436369] zswap: loaded using pool zstd

### Set parameters

#### Using sysfs

Each setting can be changed at runtime via the [sysfs](https://en.wikipedia.org/wiki/sysfs "wikipedia:sysfs") interface. For example, to change the `compressor` parameter:

# echo lz4 > /sys/module/zswap/parameters/compressor

#### Using kernel boot parameters

To persist the parameter change, the corresponding option, for example `zswap.compressor=lz4`, must be added to the kernel boot parameter. Therefore to set permanently all the above settings, the following [kernel parameters](https://wiki.archlinux.org/title/Kernel_parameters "Kernel parameters") must be added:

zswap.enabled=1 zswap.shrinker_enabled=1 zswap.compressor=lz4 zswap.max_pool_percent=30

When changing the compression algorithm via a boot parameter, ensure the corresponding compression module is loaded early during boot (refer to [#Compression algorithm](https://wiki.archlinux.org/title/Zswap#Compression_algorithm)).

### Maximum pool size

The memory pool is not preallocated, it is allowed to grow up to a certain limit in percentage of the total memory available, by default up to 20% of the total RAM. Once this threshold is reached, pages are evicted from the pool into the swap device. The maximum compressed pool size is controlled with the parameter `max_pool_percent`.

### Shrinker

The shrinker, when enabled, causes zswap to shrink the pool by evicting cold pages to swap when memory pressure is high. This reduces the amount of cold data in the pool and, [in the author's synthetic benchmark](https://github.com/torvalds/linux/commit/b5ba474f3f518701249598b35c581b92a3c95b48), helps avoid wasting CPU time on compressing and decompressing cold pages. It can be turned on with the parameter `shrinker_enabled`.

### Compression algorithm

For page compression, _zswap_ uses compressor modules provided by the kernel's cryptographic API. In official kernels the _zstd_ compression algorithm is used by default but this can be changed with `zswap.compressor=` at boot time. Other options include _deflate_, _lzo_, _842_, _lz4_ and _lz4hc_.

There is no issue changing the compression at runtime using _sysfs_ but _zswap_ starts in this case with _zstd_ and switches at a later stage to the defined algorithm. To start _zswap_ with another algorithm straight away, this must be set via the kernel boot parameters and the corresponding module must be loaded early by the kernel. This can be achieved by following these steps:

1.   Add the modules required for the chosen compressor to the [mkinitcpio#MODULES](https://wiki.archlinux.org/title/Mkinitcpio#MODULES "Mkinitcpio") array.
2.   [Regenerate the initramfs](https://wiki.archlinux.org/title/Regenerate_the_initramfs "Regenerate the initramfs").
3.   Set the compression algorithm using the `zswap.compressor=`[kernel parameter](https://wiki.archlinux.org/title/Kernel_parameter "Kernel parameter").

On next boot, see [#Current parameters](https://wiki.archlinux.org/title/Zswap#Current_parameters) to check if _zswap_ now uses the requested compressor.

### Disable writeback

_zswap_ has a per-[cgroup](https://wiki.archlinux.org/title/Cgroup "Cgroup") option to disable writeback (i.e. to prevent writes to disk).

See [Power management/Suspend and hibernate#Disable zswap writeback to use the swap space only for hibernation](https://wiki.archlinux.org/title/Power_management/Suspend_and_hibernate#Disable_zswap_writeback_to_use_the_swap_space_only_for_hibernation "Power management/Suspend and hibernate") for an example use case.

## Zswap statistics

To see _zswap_ statistics you can run this:

# grep -r . /sys/kernel/debug/zswap//sys/kernel/debug/zswap/same_filled_pages:26274
/sys/kernel/debug/zswap/stored_pages:159898
/sys/kernel/debug/zswap/pool_total_size:171565056
/sys/kernel/debug/zswap/written_back_pages:787323
/sys/kernel/debug/zswap/reject_compress_poor:0
/sys/kernel/debug/zswap/reject_compress_fail:15860
/sys/kernel/debug/zswap/reject_kmemcache_fail:0
/sys/kernel/debug/zswap/reject_alloc_fail:0
/sys/kernel/debug/zswap/reject_reclaim_fail:31
/sys/kernel/debug/zswap/pool_limit_hit:0

## See also

*   Chris Down's [Debunking zswap and zram myths](https://chrisdown.name/2026/03/24/zswap-vs-zram-when-to-use-what.html)
*   [zswap: How to determine whether it is compressing swap pages?](https://lore.kernel.org/lkml/1674223.HVFdAhB7u5@merkaba/).
*   [IBM Support Article "New Linux zswap compression functionality" (benchmark images do not load)](https://www.ibm.com/support/pages/new-linux-zswap-compression-functionality).
*   [Ask Ubuntu: zram vs. zswap vs. zcache](https://askubuntu.com/questions/471912/zram-vs-zswap-vs-zcache-ultimate-guide-when-to-use-which-one). (zcache is deprecated)
*   [Arch Linux forum thread](https://bbs.archlinux.org/viewtopic.php?id=169585).
*   [LWN.net technical article by the main developer of zswap](https://lwn.net/Articles/537422/).
