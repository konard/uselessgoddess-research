Title: libvirt: Domain XML format

URL Source: https://libvirt.org/formatdomain.html

Published Time: Fri, 27 Mar 2026 13:02:04 GMT

Markdown Content:
Domain XML format

Element and attribute overview

General metadata

Operating system booting

Guest firmware

Host bootloader

Direct kernel boot

Container boot

Common <os> element configuration

SMBIOS System Information

CPU Allocation

IOThreads Allocation

CPU Tuning

Memory Allocation

Memory Backing

Memory Tuning

NUMA Node Tuning

Block I/O Tuning

Host Device IOMMUFD

Resource partitioning

Fibre Channel VMID

CPU model and topology

ACPI Heterogeneous Memory Attribute Table

Events configuration

Power Management

Disk Throttle Group Management

Hypervisor features

Time keeping

Performance monitoring events

Devices

Hard drives, floppy disks, CDROMs

Filesystems

Device Addresses

Virtio-related options

Virtio device models

Controllers

Device leases

Host device assignment

USB / PCI / SCSI devices

ACPI Generic Initiators

Block / character devices

Redirected devices

Smartcard devices

Network interfaces

Virtual network

Bridge to LAN

Userspace connection using SLIRP

Userspace connection using passt

Generic ethernet connection

Direct attachment to physical interface

PCI Passthrough

vDPA devices

Teaming a virtio/hostdev NIC pair

Multicast tunnel

TCP tunnel

UDP unicast tunnel

Null network interface

VMWare Distributed Switch

Setting the NIC model

Setting NIC driver-specific options

Setting network backend-specific options

Overriding the target element

Specifying boot order

Interface ROM BIOS configuration

Setting up a network backend in a driver domain

Quality of service

Setting VLAN tag (on supported network types only)

Isolating guests' network traffic from each other

Modifying virtual link state

MTU configuration

Coalesce settings

IP configuration

vhost-user connection

vhost-user connection with passt backend

Traffic filtering with NWFilter

Input devices

Hub devices

Graphical framebuffers

Video devices

Consoles, serial, parallel & channel devices

Guest interface

Parallel port

Serial port

Console

Relationship between serial ports and consoles

Channel

Host interface

Domain logfile

Device logfile

Virtual console

Null device

Pseudo TTY

Host device proxy

Named pipe

TCP client/server

UDP network console

UNIX domain socket client/server

Spice channel

Nmdm device

Sound devices

Audio backends

None audio backend

ALSA audio backend

Coreaudio audio backend

D-Bus audio backend

Jack audio backend

OSS audio backend

PipeWire audio backend

PulseAudio audio backend

SDL audio backend

Spice audio backend

File audio backend

Watchdog devices

Memory balloon device

Random number generator device

TPM device

NVRAM device

panic device

Shared memory device

Memory devices

IOMMU devices

Vsock

Crypto

Pstore

Security label

Key Wrap

Launch Security

Example configs

This section describes the XML format used to represent domains, there are variations on the format based on the kind of domains run and the options used to launch them. For hypervisor specific details consult the driver docs

Element and attribute overview

The root element required for all virtual machines is named domain. It has two attributes, the type specifies the hypervisor used for running the domain. The allowed values are driver specific, but include "xen", "kvm", "hvf" (since 8.1.0 and QEMU 2.12), "qemu" and "lxc". The second attribute is id which is a unique integer identifier for the running guest machine. Inactive machines have no id value.

General metadata
<domain type='kvm' id='1'>
  <name>MyGuest</name>
  <uuid>4dea22b3-1d52-d8f3-2516-782e98ab3fa0</uuid>
  <genid>43dc0cf8-809b-4adb-9bea-a9abb5f3d90e</genid>
  <title>A short description - title - of the domain</title>
  <description>Some human readable description</description>
  <metadata>
    <app1:foo xmlns:app1="http://app1.org/app1/">..</app1:foo>
    <app2:bar xmlns:app2="http://app1.org/app2/">..</app2:bar>
  </metadata>
  ...
name

The content of the name element provides a short name for the virtual machine. This name should consist only of alphanumeric characters and is required to be unique within the scope of a single host. It is often used to form the filename for storing the persistent configuration file. Since 0.0.1

uuid

The content of the uuid element provides a globally unique identifier for the virtual machine. The format must be RFC 4122 compliant, eg 3e3fce45-4f53-4fa7-bb32-11f34168b82b. If omitted when defining/creating a new machine, a random UUID is generated. Since 0.0.1

Since 0.8.7, it is also possible to provide the UUID via a SMBIOS System Information specification.

hwuuid

The optional hwuuid element can be used to supply an alternative UUID for identifying the virtual machine from the domain uuid above. The difference between using the hwuuid element and simply providing an alternative UUID via a SMBIOS System Information specification is that the hwuuid affects all devices that expose the UUID to the guest. Since 11.7.0 QEMU/KVM only

genid

Since 4.4.0, the genid element can be used to add a Virtual Machine Generation ID which exposes a 128-bit, cryptographically random, integer value identifier, referred to as a Globally Unique Identifier (GUID) using the same format as the uuid. The value is used to help notify the guest operating system when the virtual machine is re-executing something that has already executed before, such as:

VM starts executing a snapshot

VM is recovered from backup

VM is failover in a disaster recovery environment

VM is imported, copied, or cloned

The guest operating system notices the change and is then able to react as appropriate by marking its copies of distributed databases as dirty, re-initializing its random number generator, etc.

The libvirt XML parser will accept both a provided GUID value or just <genid/> in which case a GUID will be generated and saved in the XML. For the transitions such as above, libvirt will change the GUID before re-executing.

title

The optional element title provides space for a short description of the domain. The title should not contain any newlines. Since 0.9.10.

description

The content of the description element provides a human readable description of the virtual machine. This data is not used by libvirt in any way, it can contain any information the user wants. Since 0.7.2

metadata

The metadata node can be used by applications to store custom metadata in the form of XML nodes/trees. Applications must use custom namespaces on their XML nodes/trees, with only one top-level element per namespace (if the application needs structure, they should have sub-elements to their namespace element). Since 0.9.10

Operating system booting

There are a number of different ways to boot virtual machines each with their own pros and cons.

Guest firmware

Booting via a guest firmware is available for hypervisors supporting full virtualization. In this case the firmware has a boot order priority (floppy, harddisk, cdrom, network) determining where to obtain/find the boot image.

<!-- Xen with fullvirt loader -->
...
<os>
  <type>hvm</type>
  <loader>/usr/lib/xen/boot/hvmloader</loader>
  <boot dev='hd'/>
</os>
...

<!-- QEMU with default firmware, serial console and SMBIOS -->
...
<os>
  <type>hvm</type>
  <boot dev='cdrom'/>
  <bootmenu enable='yes' timeout='3000'/>
  <smbios mode='sysinfo'/>
  <bios useserial='yes' rebootTimeout='0'/>
</os>
...

<!-- QEMU with UEFI manual firmware and secure boot -->
...
<os>
  <type>hvm</type>
  <loader readonly='yes' secure='yes' type='pflash'>/usr/share/OVMF/OVMF_CODE.fd</loader>
  <nvram template='/usr/share/OVMF/OVMF_VARS.fd'>/var/lib/libvirt/nvram/guest_VARS.fd</nvram>
  <boot dev='hd'/>
</os>
...

<!-- QEMU with UEFI manual firmware, secure boot and with NVRAM type 'file'-->
...
<os>
  <type>hvm</type>
  <loader readonly='yes' secure='yes' type='pflash'>/usr/share/OVMF/OVMF_CODE.fd</loader>
  <nvram type='file' template='/usr/share/OVMF/OVMF_VARS.fd'>
    <source file='/var/lib/libvirt/nvram/guest_VARS.fd'/>
  </nvram>
  <boot dev='hd'/>
</os>
...

<!-- QEMU with UEFI manual firmware, secure boot and with network backed NVRAM'-->
...
<os>
  <type>hvm</type>
  <loader readonly='yes' secure='yes' type='pflash'>/usr/share/OVMF/OVMF_CODE.fd</loader>
  <nvram type='network'>
    <source protocol='iscsi' name='iqn.2013-07.com.example:iscsi-nopool/0'>
      <host name='example.com' port='6000'/>
      <auth username='myname'>
        <secret type='iscsi' usage='mycluster_myname'/>
      </auth>
    </source>
  </nvram>
  <boot dev='hd'/>
</os>
...

<!-- QEMU with automatic UEFI firmware and secure boot -->
...
<os firmware='efi'>
  <type>hvm</type>
  <loader secure='yes'/>
  <boot dev='hd'/>
</os>
...

<!-- QEMU with automatic UEFI stateless firmware for AMD SEV -->
...
<os firmware='efi'>
  <type>hvm</type>
  <loader stateless='yes'/>
  <boot dev='hd'/>
</os>
...
firmware

The firmware attribute allows management applications to automatically fill <loader/> and <nvram/> or <varstore/> elements and possibly enable some features required by selected firmware. Accepted values are bios and efi. The selection process scans for files describing installed firmware images in specified location and uses the most specific one which fulfills domain requirements. The locations in order of preference (from generic to most specific one) are:

/usr/share/qemu/firmware

/etc/qemu/firmware

$XDG_CONFIG_HOME/qemu/firmware

For more information refer to firmware metadata specification as described in docs/interop/firmware.json in QEMU repository. Regular users do not need to bother. Since 5.2.0 (QEMU and KVM only) For VMware guests, this is set to efi when the guest uses UEFI, and it is not set when using BIOS. Since 5.3.0 (VMware ESX and Workstation/Player)

type

The content of the type element specifies the type of operating system to be booted in the virtual machine. hvm indicates that the OS is one designed to run on bare metal, so requires full virtualization. linux (badly named!) refers to an OS that supports the Xen 3 hypervisor guest ABI. There are also two optional attributes, arch specifying the CPU architecture to virtualization, and machine referring to the machine type. The Capabilities XML provides details on allowed values for these. If arch is omitted then for most hypervisor drivers, the host native arch will be chosen. For the test, ESX and VMWare hypervisor drivers, however, the i686 arch will always be chosen even on an x86_64 host. Since 0.0.1

firmware

Since 7.2.0 QEMU/KVM only

When using firmware auto-selection there are different features enabled in the firmwares. The list of features can be used to limit what firmware should be automatically selected for the VM. The list of features can be specified using zero or more feature elements. Libvirt will take into consideration only the listed features and ignore the rest when selecting the firmware.

feature

The list of mandatory attributes:

enabled (accepted values are yes and no) is used to tell libvirt if the feature must be enabled or not in the automatically selected firmware

name the name of the feature, the list of the features:

enrolled-keys whether the selected nvram template has default certificate enrolled. Firmware with Secure Boot feature but without enrolled keys will successfully boot non-signed binaries as well. Valid only for firmwares with Secure Boot feature.

secure-boot whether the firmware implements UEFI Secure boot feature.

loader

The optional loader tag refers to a firmware blob, which is specified by absolute path, used to assist the domain creation process. It is used by Xen fully virtualized domains as well as setting the QEMU BIOS file path for QEMU/KVM domains. Xen since 0.1.0, QEMU/KVM since 0.9.12 Then, since 1.2.8 it's possible for the element to have two optional attributes: readonly (accepted values are yes and no) to reflect the fact that the image should be writable or read-only. The second attribute type accepts values rom and pflash. It tells the hypervisor where in the guest memory the file should be mapped. For instance, if the loader path points to an UEFI image, type should be pflash. Moreover, some firmwares may implement the Secure boot feature. Attribute secure can be used to tell the hypervisor that the firmware is capable of Secure Boot feature. It cannot be used to enable or disable the feature itself in the firmware. Since 2.1.0. If the loader is marked as read-only, then with UEFI it is assumed that there will be a writable NVRAM available. In some cases, however, it may be desirable for the loader to run without any NVRAM, discarding any config changes on shutdown. The stateless flag (Since 8.6.0) can be used to control this behaviour, when set to yes NVRAM will never be created.

When firmware autoselection is enabled, the format attribute can be used to tell libvirt to only consider firmware builds that are in a specific format. Supported values are raw and qcow2. Since 9.2.0 (QEMU only)

nvram

Some UEFI firmwares may want to use a non-volatile memory to store some variables. In the host, this is represented as a file and the absolute path to the file is stored in this element. Moreover, when the domain is started up libvirt copies so called master NVRAM store file either selected by the firmware autoselection process or defined in qemu.conf. If needed, the template attribute can be used to override the automatically chosen NVRAM template and templateFormat to specify the format for the template file (currently supported are raw and qcow2). When firmware auto-selection is in use the templateFormat field reflects the format of the picked template. Since 10.10.0 (QEMU only)

Note, that for transient domains if the NVRAM file has been created by libvirt it is left behind and it is management application's responsibility to save and remove file (if needed to be persistent). Since 1.2.8

Since 8.5.0, it's possible for the element to have type attribute (accepts values file, block and network) in that case the NVRAM storage is described by a <source> sub-element with the same syntax as disk's source. See Hard drives, floppy disks, CDROMs. For block backed NVRAM images it may be necessary to ensure that the block device has the correct guest visible size based on hypervisor expectations. This may require use of non raw format image that allows arbitrary disk size.

Note: network backed NVRAM the variables are not instantiated from the template and it's user's responsibility to provide a valid NVRAM image.

This element supports a format attribute, which specifies the format of the NVRAM image. Since 9.2.0 (QEMU only) Note that hypervisors may not support automatic population of the nvram if format differs from templateFormat or may support only a specific format.

It is not valid to provide this element if the loader is marked as stateless.

varstore

This works much the same way as the <nvram/> element described above, except that variable storage is handled by the uefi-vars QEMU device instead of being backed by a pflash device. Since 12.1.0 (QEMU only)

The path attribute contains the path of the domain-specific file where variables are stored, while the template attribute points to a template that the domain-specific file can be (re)generated from. Assuming that the necessary JSON firmware descriptor files are present, both attributes will be filled in automatically by libvirt.

Using <varstore/> instead of <nvram/> is particularly useful on non-x86 architectures such as aarch64, where it represents the only way to get Secure Boot working. It can be used on x86 too, and doing so will make it possible to keep UEFI authenticated variables safe from tampering without requiring the use of SMM emulation.

boot

The dev attribute takes one of the values "fd", "hd", "cdrom" or "network" and is used to specify the next boot device to consider. The boot element can be repeated multiple times to setup a priority list of boot devices to try in turn. Multiple devices of the same type are sorted according to their targets while preserving the order of buses. After defining the domain, its XML configuration returned by libvirt (through virDomainGetXMLDesc) lists devices in the sorted order. Once sorted, the first device is marked as bootable. Thus, e.g., a domain configured to boot from "hd" with vdb, hda, vda, and hdc disks assigned to it will boot from vda (the sorted list is vda, vdb, hda, hdc). Similar domain with hdc, vda, vdb, and hda disks will boot from hda (sorted disks are: hda, hdc, vda, vdb). It can be tricky to configure in the desired way, which is why per-device boot elements (see Hard drives, floppy disks, CDROMs, Network interfaces, and Host device assignment sections below) were introduced and they are the preferred way providing full control over booting order. The boot element and per-device boot elements are mutually exclusive. Since 0.1.3, per-device boot since 0.8.8

smbios

How to populate SMBIOS information visible in the guest. The mode attribute must be specified, and is either "emulate" (let the hypervisor generate all values), "host" (copy all of Block 0 and Block 1, except for the UUID, from the host's SMBIOS values; the virConnectGetSysinfo call can be used to see what values are copied), or "sysinfo" (use the values in the SMBIOS System Information element). If not specified, the hypervisor default is used. Since 0.8.7

Up till here the BIOS/UEFI configuration knobs are generic enough to be implemented by majority (if not all) firmwares out there. However, from now on not every single setting makes sense to all firmwares. For instance, rebootTimeout doesn't make sense for UEFI, useserial might not be usable with a BIOS firmware that doesn't produce any output onto serial line, etc. Moreover, firmwares don't usually export their capabilities for libvirt (or users) to check. And the set of their capabilities can change with every new release. Hence users are advised to try the settings they use before relying on them in production.

bootmenu

Whether or not to enable an interactive boot menu prompt on guest startup. The enable attribute can be either "yes" or "no". If not specified, the hypervisor default is used. Since 0.8.3 Additional attribute timeout takes the number of milliseconds the boot menu should wait until it times out. Allowed values are numbers in range [0, 65535] inclusive and it is ignored unless enable is set to "yes". Since 1.2.8

bios

This element has attribute useserial with possible values yes or no. It enables or disables Serial Graphics Adapter which allows users to see BIOS messages on a serial port. Therefore, one needs to have Serial port defined. Since 0.9.4. The rebootTimeout attribute (since 0.10.2 (QEMU only)) controls whether and after how long the guest should start booting again in case the boot fails (according to BIOS). The value is in milliseconds with maximum of 65535 and special value -1 disables the reboot.

Host bootloader

Hypervisors employing paravirtualization do not usually emulate a BIOS, and instead the host is responsible to kicking off the operating system boot. This may use a pseudo-bootloader in the host to provide an interface to choose a kernel for the guest. An example is pygrub with Xen. The Bhyve hypervisor also uses a host bootloader, either bhyveload or grub-bhyve.

...
<bootloader>/usr/bin/pygrub</bootloader>
<bootloader_args>--append single</bootloader_args>
...
bootloader

The content of the bootloader element provides a fully qualified path to the bootloader executable in the host OS. This bootloader will be run to choose which kernel to boot. The required output of the bootloader is dependent on the hypervisor in use. Since 0.1.0

bootloader_args
