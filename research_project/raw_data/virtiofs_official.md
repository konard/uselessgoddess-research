Title: shared file system for virtual machines

URL Source: https://virtio-fs.gitlab.io/

Published Time: Thu, 18 Apr 2024 17:16:28 GMT

Markdown Content:
# virtiofs - shared file system for virtual machines

[Overview](https://virtio-fs.gitlab.io/index.html#overview)[Status](https://virtio-fs.gitlab.io/index.html#status)[HowTo](https://virtio-fs.gitlab.io/index.html#howto)[FAQ](https://virtio-fs.gitlab.io/index.html#faq)

![Image 1](https://virtio-fs.gitlab.io/logo.png)

# Overview

Virtiofs is a shared file system that lets virtual machines access a directory tree on the host. Unlike existing approaches, it is designed to offer local file system semantics and performance.

Virtiofs was started at Red Hat and is being developed in the Linux, QEMU, FUSE, and Kata Containers open source communities.

See the [design document](https://virtio-fs.gitlab.io/design.html) for a more in-depth explanation of virtiofs.

# Status

Available in mainline since [Linux](https://kernel.org/) 5.4, [QEMU](https://qemu.org/) 5.0, [libvirt](https://libvirt.org/) 6.2, and [Kata Containers](https://katacontainers.io/) 1.7.

The new [virtiofsd-rs](https://gitlab.com/virtio-fs/virtiofsd) Rust daemon is receiving the most attention for new feature development.

# Community

Chat: [#virtiofs on Matrix](https://matrix.to/#/#virtiofs:matrix.org)

Mailing list: [virtio-fs@lists.linux.dev](mailto:virtio-fs@lists.linux.dev.com) ([list info](https://lists.linux.dev/))

Community call: Bi-weekly on Wednesdays via [video conference](https://bluejeans.com/318831955) or [phone](https://www.redhat.com/en/conference-numbers) (meeting ID 318831955). [Meeting times and agenda](https://etherpad.opendev.org/p/virtiofs-external-meeting).

# HowTo

*   [Sharing files with virtiofs using libvirt](https://libvirt.org/kbase/virtiofs.html)
*   [Installing virtiofs drivers on Windows](https://virtio-fs.gitlab.io/howto-windows.html)
*   [Kata Containers with virtiofs](https://github.com/kata-containers/documentation/blob/master/how-to/how-to-use-virtio-fs-with-kata.md)
*   [Booting from virtiofs](https://virtio-fs.gitlab.io/howto-boot.html)
*   [Manually running QEMU with virtiofs](https://virtio-fs.gitlab.io/howto-qemu.html)

# Frequently Asked Questions

#### Why virtiofs?

Virtiofs is not a network file system repurposed for virtualization, it is specifically designed to take advantage of the locality of virtual machines and the hypervisor.

The goal of virtiofs is to provide local file system semantics between multiple virtual machines sharing a directory tree. This is especially useful for lightweight VMs and container workloads, where shared volumes are a requirement.

#### Why a new file system?

Existing solutions to this problem, such as virtio-9p, are based on existing network protocols that are not optimized for virtualization use cases. As a result they do not perform as well as local file systems and do not provide the semantics that some applications rely on.

Virtiofs takes advantage of the virtual machine’s co-location with the hypervisor to avoid overheads associated with network file systems.

#### How does it work?

Virtiofs uses [FUSE](https://github.com/libfuse/libfuse) as the foundation. Unlike traditional FUSE where the file system daemon runs in userspace, the virtiofs daemon runs on the host. A [VIRTIO](https://www.oasis-open.org/committees/tc_home.php?wg_abbrev=virtio) device carries FUSE messages and provides extensions for advanced features not available in traditional FUSE.

FUSE has no dependencies on a networking stack and exposes a rich native Linux file system interface that allows virtiofs to act like a local file system.

#### How does virtiofs exploit Direct Access (DAX)? [experimental]

File contents can be mapped into a memory window on the host, allowing the guest to directly access data from the host page cache. This has several advantages:

*   The guest page cache is bypassed, reducing the memory footprint.
*   No communication is necessary to access file contents, improving I/O performance.
*   Shared file access is coherent between virtual machines on the same host even with mmap.

#### How does virtiofs support coherence for metadata? [experimental]

A metadata version table in shared memory provides coherent access for multiple virtual machines without expensive communication. Version numbers for file system metadata are implemented using atomic compare-and-swap operations. Virtual machines refresh metadata when the version number has changed since they last cached a copy.

This website is published under the [Creative Commons Attribution-ShareAlike 4.0 International](https://creativecommons.org/licenses/by-sa/4.0/) license. The source code is available [here](https://gitlab.com/virtio-fs/virtio-fs.gitlab.io/).
