Title: Virtio-GPU Venus — The Mesa 3D Graphics Library latest documentation

URL Source: https://docs.mesa3d.org/drivers/venus.html

Published Time: Mon, 30 Mar 2026 07:57:00 GMT

Markdown Content:
# Virtio-GPU Venus — The Mesa 3D Graphics Library latest documentation

[![Image 1](https://docs.mesa3d.org/_static/logo.svg) Mesa 3D](https://www.mesa3d.org/)

*   [Home](https://www.mesa3d.org/ "Home")
*   [News](https://www.mesa3d.org/news/ "News")
*   [Getting Started](https://docs.mesa3d.org/download.html)
*   [Documentation](https://docs.mesa3d.org/index.html)

# Virtio-GPU Venus[¶](https://docs.mesa3d.org/drivers/venus.html#virtio-gpu-venus "Link to this heading")

Venus is a Virtio-GPU protocol for Vulkan command serialization. The protocol definition and codegen are hosted at [venus-protocol](https://gitlab.freedesktop.org/virgl/venus-protocol). The renderer is hosted at [virglrenderer](https://gitlab.freedesktop.org/virgl/virglrenderer).

## Requirements[¶](https://docs.mesa3d.org/drivers/venus.html#requirements "Link to this heading")

The Venus renderer requires

*   Linux platform
    *   Vulkan 1.1

    *   [VK_KHR_external_memory_fd](https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_external_memory_fd.html)

*   Android platform
    *   Vulkan 1.1

    *   [VK_EXT_external_memory_dma_buf](https://docs.vulkan.org/refpages/latest/refpages/source/VK_EXT_external_memory_dma_buf.html)

    *   [VK_EXT_image_drm_format_modifier](https://docs.vulkan.org/refpages/latest/refpages/source/VK_EXT_image_drm_format_modifier.html)

    *   [VK_EXT_queue_family_foreign](https://docs.vulkan.org/refpages/latest/refpages/source/VK_EXT_queue_family_foreign.html)

from the host driver. However, it violates the spec and relies on implementation-defined behaviors to support `vkMapMemory` (see [below](https://docs.mesa3d.org/drivers/venus.html#vk-memory-property-host-visible-bit)). It is not expected to work on all drivers meeting the requirements. It has only been tested with:

*   ANV 21.1 or later
    *   Note: with Intel Meteor Lake or xe driver, you need 6.16+ kernel and 11.0+ QEMU with `-accel kvm,honor-guest-pat=on` (request to default that on is [here](https://gitlab.com/qemu-project/qemu/-/work_items/3357)).

*   RADV 21.1 or later
    *   Note: you need 6.13+ kernel that already has [KVM: Stop grabbing references to PFNMAP’d pages](https://lore.kernel.org/all/20241010182427.1434605-1-seanjc@google.com/).

    *   Note: for dGPU paired with Intel CPU, you need 6.16+ kernel and 11.0+ QEMU with `-accel kvm,honor-guest-pat=on` (request to default that on is [here](https://gitlab.com/qemu-project/qemu/-/work_items/3357)).

*   NVIDIA (Proprietary) 570.86 or later
    *   Note: if paired with Intel CPU, you need 6.16+ kernel and 11.0+ QEMU with `-accel kvm,honor-guest-pat=on` (request to default that on is [here](https://gitlab.com/qemu-project/qemu/-/work_items/3357)).

*   ARM Mali (Proprietary) r32p0 or later

*   Turnip 22.0 or later

*   PanVK 25.1 or later

*   Lavapipe 22.1 or later

The Venus driver requires supports for

*   `VIRTGPU_PARAM_3D_FEATURES`

*   `VIRTGPU_PARAM_CAPSET_QUERY_FIX`

*   `VIRTGPU_PARAM_RESOURCE_BLOB`

*   `VIRTGPU_PARAM_HOST_VISIBLE`

*   `VIRTGPU_PARAM_CONTEXT_INIT`

from the virtio-gpu kernel driver, unless vtest is used. That usually means the guest kernel should be at least 5.16 or have the parameters back ported, paired with hypervisors such as [crosvm](https://crosvm.dev/), or [QEMU](https://www.qemu.org/).

## vtest[¶](https://docs.mesa3d.org/drivers/venus.html#vtest "Link to this heading")

The simplest way to test Venus is to use virglrenderer’s vtest server. To build virglrenderer with Venus support and to start the vtest server,

$ git clone https://gitlab.freedesktop.org/virgl/virglrenderer.git
$ cd virglrenderer
$ meson out -Dvenus=true
$ meson compile -C out
$ meson devenv -C out
$ ./vtest/virgl_test_server --venus
$ exit

In another shell,

$ export VK_DRIVER_FILES=<path-to-virtio_icd.x86_64.json>
$ export VN_DEBUG=vtest
$ vulkaninfo
$ vkcube

If the host driver of the system is not new enough, it is a good idea to build the host driver as well when building the Venus driver. Just remember to set [`VK_DRIVER_FILES`](https://docs.mesa3d.org/envvars.html#envvar-VK_DRIVER_FILES) when starting the vtest server so that the vtest server finds the locally built host driver.

## QEMU[¶](https://docs.mesa3d.org/drivers/venus.html#qemu "Link to this heading")

This is how one might want to start QEMU

$ ./qemu-system-x86_64 \
 -enable-kvm \
 -M q35 \
 -smp 8 \
 -m 4G \
 -cpu host \
 -net nic,model=virtio \
 -net user,hostfwd=tcp::2222-:22 \
 -device virtio-gpu-gl,hostmem=4G,blob=true,venus=true \
 -vga none \
 -display sdl,gl=on,show-cursor=on \
 -usb -device usb-tablet \
 -object memory-backend-memfd,id=mem1,size=4G \
 -machine memory-backend=mem1 \
 -hda $IMG

To build QEMU, this is how one might want to configure it

$ cd <QEMU source dir>
$ mkdir build && cd build
$ ../configure \
 --prefix=$HOME/.local \
 --target-list=x86_64-softmmu \
 --enable-kvm \
 --disable-werror \
 --enable-opengl \
 --enable-virglrenderer \
 --enable-gtk \
 --enable-sdl
$ make -j$(nproc)

## crosvm[¶](https://docs.mesa3d.org/drivers/venus.html#crosvm "Link to this heading")

crosvm is written in Rust. To build crosvm, make sure Rust has been installed and

$ git clone --recurse-submodules \
 https://chromium.googlesource.com/chromiumos/platform/crosvm
$ cd crosvm
$ RUSTFLAGS="-L<path-to-virglrenderer>/out/src" cargo build \
 --features "x wl-dmabuf virgl_renderer virgl_renderer_next default-no-sandbox"

Note that crosvm must be built with `default-no-sandbox` or started with `--disable-sandbox` in this setup.

This is how one might want to start crosvm

$ sudo LD_LIBRARY_PATH=<...> VK_DRIVER_FILES=<...> ./target/debug/crosvm run \
 --gpu vulkan=true \
 --gpu-render-server path=<path-to-virglrenderer>/out/server/virgl_render_server \
 --display-window-keyboard \
 --display-window-mouse \
 --net "host-ip 192.168.0.1,netmask=255.255.255.0,mac=12:34:56:78:9a:bc" \
 --rwdisk disk.img \
 -p root=/dev/vda1 \
 <path-to-bzImage>

assuming a working system is installed to partition 1 of `disk.img`. `sudo` or `CAP_NET_ADMIN` is needed to set up the TAP network device.

## Android Cuttlefish[¶](https://docs.mesa3d.org/drivers/venus.html#android-cuttlefish "Link to this heading")

Venus isn’t supported in the upstream Cuttlefish yet, for `venus_guest_angle` mode used in Mesa CI against Android 16 AOSP, the instruction is [here](https://gitlab.freedesktop.org/gfx-ci/android/aosp-manifest/-/blob/android16-release+venus/README.md).

## Optional Requirements[¶](https://docs.mesa3d.org/drivers/venus.html#optional-requirements "Link to this heading")

In the future, if virglrenderer’s `virgl_renderer_export_fence` is supported, the Venus renderer will require [VK_KHR_external_fence_fd](https://docs.vulkan.org/refpages/latest/refpages/source/VK_KHR_external_fence_fd.html) with `VK_EXTERNAL_FENCE_HANDLE_TYPE_SYNC_FD_BIT` from the host driver.

## VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT[¶](https://docs.mesa3d.org/drivers/venus.html#vk-memory-property-host-visible-bit "Link to this heading")

The Venus renderer makes assumptions about `VkDeviceMemory` that has `VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT`. The assumptions are illegal and rely on the current behaviors of the host drivers. It should be possible to remove some of the assumptions and incrementally improve compatibilities with more host drivers by imposing platform-specific requirements. But the long-term plan is to create a new Vulkan extension for the host drivers to address this specific use case.

The Venus renderer assumes a device memory that has `VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT` can be exported as a mmapable dma-buf (in the future, the plan is to export the device memory as an opaque fd). It chains `VkExportMemoryAllocateInfo` to `VkMemoryAllocateInfo` without checking if the host driver can export the device memory.

The dma-buf is mapped (in the future, the plan is to import the opaque fd and call `vkMapMemory`) but the mapping is not accessed. Instead, the mapping is passed to `KVM_SET_USER_MEMORY_REGION`. The hypervisor, host KVM, and the guest kernel work together to set up a write-back or write-combined guest mapping (see `virtio_gpu_vram_mmap` of the virtio-gpu kernel driver). CPU accesses to the device memory are via the guest mapping, and are assumed to be coherent when the device memory also has `VK_MEMORY_PROPERTY_HOST_COHERENT_BIT`.

While the Venus renderer can force a `VkDeviceMemory` external, it does not force a `VkImage` or a `VkBuffer` external. As a result, it can bind an external device memory to a non-external resource.

Documentation

*   [Introduction](https://docs.mesa3d.org/index.html)
*   [Project History](https://docs.mesa3d.org/history.html)
*   [Amber Branch](https://docs.mesa3d.org/amber.html)
*   [Platforms and Drivers](https://docs.mesa3d.org/systems.html)
*   [License and Copyright](https://docs.mesa3d.org/license.html)
*   [Frequently Asked Questions](https://docs.mesa3d.org/faq.html)
*   [Release Notes](https://docs.mesa3d.org/relnotes.html)

Download and Install

*   [Downloading and Unpacking](https://docs.mesa3d.org/download.html)
*   [Compiling and Installing](https://docs.mesa3d.org/install.html)
*   [Precompiled Libraries](https://docs.mesa3d.org/precompiled.html)

Need help?

*   [Mailing Lists](https://docs.mesa3d.org/lists.html)
*   [Report a Bug](https://docs.mesa3d.org/bugs.html)

User Topics

*   [Shading Language](https://docs.mesa3d.org/glsl.html)
*   [EGL](https://docs.mesa3d.org/egl.html)
*   [OpenGL ES](https://docs.mesa3d.org/opengles.html)
*   [Environment Variables](https://docs.mesa3d.org/envvars.html)
*   [Performance Tips](https://docs.mesa3d.org/perf.html)
*   [GPU Performance Tracing](https://docs.mesa3d.org/gpu-perf-tracing.html)
*   [Mesa Extensions](https://docs.mesa3d.org/extensions.html)
*   [Application Issues](https://docs.mesa3d.org/application-issues.html)
*   [Viewperf Issues](https://docs.mesa3d.org/viewperf.html)
*   [TensorFlow Lite delegate](https://docs.mesa3d.org/teflon.html)

Drivers

*   [ANV](https://docs.mesa3d.org/drivers/anv.html)
*   [Asahi](https://docs.mesa3d.org/drivers/asahi.html)
*   [D3D12](https://docs.mesa3d.org/drivers/d3d12.html)
*   [Freedreno](https://docs.mesa3d.org/drivers/freedreno.html)
*   [KosmicKrisp](https://docs.mesa3d.org/drivers/kosmickrisp.html)
*   [Lima](https://docs.mesa3d.org/drivers/lima.html)
*   [LLVMpipe](https://docs.mesa3d.org/drivers/llvmpipe.html)
*   [NVK](https://docs.mesa3d.org/drivers/nvk.html)
*   [Panfrost](https://docs.mesa3d.org/drivers/panfrost.html)
*   [PowerVR](https://docs.mesa3d.org/drivers/powervr.html)
*   [RADV](https://docs.mesa3d.org/drivers/radv.html)
*   [VMware SVGA3D](https://docs.mesa3d.org/drivers/svga3d.html)
*   [V3D](https://docs.mesa3d.org/drivers/v3d.html)
*   [VC4](https://docs.mesa3d.org/drivers/vc4.html)
*   [Virtio-GPU Venus](https://docs.mesa3d.org/drivers/venus.html#)
    *   [Requirements](https://docs.mesa3d.org/drivers/venus.html#requirements)
    *   [vtest](https://docs.mesa3d.org/drivers/venus.html#vtest)
    *   [QEMU](https://docs.mesa3d.org/drivers/venus.html#qemu)
    *   [crosvm](https://docs.mesa3d.org/drivers/venus.html#crosvm)
    *   [Android Cuttlefish](https://docs.mesa3d.org/drivers/venus.html#android-cuttlefish)
    *   [Optional Requirements](https://docs.mesa3d.org/drivers/venus.html#optional-requirements)
    *   [VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT](https://docs.mesa3d.org/drivers/venus.html#vk-memory-property-host-visible-bit)

*   [VirGL](https://docs.mesa3d.org/drivers/virgl.html)
*   [Zink](https://docs.mesa3d.org/drivers/zink.html)
*   [Xlib Software Driver](https://docs.mesa3d.org/xlibdriver.html)

Developer Topics

*   [Source Code Repository](https://docs.mesa3d.org/repository.html)
*   [Source Code Tree](https://docs.mesa3d.org/sourcetree.html)
*   [Development Utilities](https://docs.mesa3d.org/utilities.html)
*   [Help Wanted](https://docs.mesa3d.org/helpwanted.html)
*   [Development Notes](https://docs.mesa3d.org/devinfo.html)
*   [Coding Style](https://docs.mesa3d.org/codingstyle.html)
*   [Submitting Patches](https://docs.mesa3d.org/submittingpatches.html)
*   [Rust](https://docs.mesa3d.org/rust.html)
*   [Releasing Process](https://docs.mesa3d.org/releasing.html)
*   [Release Calendar](https://docs.mesa3d.org/release-calendar.html)
*   [Debugging GPU hangs, faults, and misrenderings](https://docs.mesa3d.org/graphics-debugging/debugging-misrenderings-crashes.html)
*   [GL Dispatch](https://docs.mesa3d.org/dispatch.html)
*   [Gallium](https://docs.mesa3d.org/gallium/index.html)
*   [Vulkan Runtime](https://docs.mesa3d.org/vulkan/index.html)
*   [NIR Intermediate Representation (NIR)](https://docs.mesa3d.org/nir/index.html)
*   [SPIR-V Debugging](https://docs.mesa3d.org/spirv/index.html)
*   [Intel Surface Layout (ISL)](https://docs.mesa3d.org/isl/index.html)
*   [ISASPEC - XML Based ISA Specification](https://docs.mesa3d.org/isaspec.html)
*   [Rusticl](https://docs.mesa3d.org/rusticl.html)
*   [Android](https://docs.mesa3d.org/android.html)
*   [Notes for macOS](https://docs.mesa3d.org/macos.html)
*   [Linux Kernel Drivers](https://www.kernel.org/doc/html/latest/gpu/)

Testing

*   [Conformance Testing](https://docs.mesa3d.org/conform.html)
*   [Continuous Integration](https://docs.mesa3d.org/ci/index.html)

Links

*   [OpenGL Website](https://www.opengl.org/)
*   [DRI Website](https://dri.freedesktop.org/)
*   [Developer Blogs](https://planet.freedesktop.org/)

###### Documentation

*   [License](https://docs.mesa3d.org/license.html)
*   [FAQ](https://docs.mesa3d.org/faq.html)
*   [Getting Started](https://docs.mesa3d.org/download.html)

###### Community

*   [GitLab](https://gitlab.freedesktop.org/mesa)
*   [Mailing Lists](https://docs.mesa3d.org/lists.html)
*   [Report a Bug](https://docs.mesa3d.org/bugs.html)

###### More

*   [About Mesa3D.org](https://www.mesa3d.org/website/)
*   [Acknowledgements](https://docs.mesa3d.org/thanks.html)
*   [Mesa / DRI Wiki](https://dri.freedesktop.org/wiki/)

Hosted by [Freedesktop.org](https://www.freedesktop.org/)

[Edit this page](https://gitlab.freedesktop.org/mesa/mesa/-/blob/main/docs/drivers/venus.rst)
