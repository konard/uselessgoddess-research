Title: QEMU - ArchWiki

URL Source: https://wiki.archlinux.org/title/QEMU

Published Time: Mon, 30 Mar 2026 03:44:17 GMT

Markdown Content:
# QEMU - ArchWiki

[](https://archlinux.org/)

*   [Home](https://archlinux.org/)
*   [Packages](https://archlinux.org/packages/)
*   [Forums](https://bbs.archlinux.org/)
*   [Wiki](https://wiki.archlinux.org/)
*   [GitLab](https://gitlab.archlinux.org/archlinux)
*   [Security](https://security.archlinux.org/)
*   [AUR](https://aur.archlinux.org/)
*   [Download](https://archlinux.org/download/)

[Jump to content](https://wiki.archlinux.org/title/QEMU#bodyContent)

- [x] Main menu 

Main menu

move to sidebar hide

 Navigation 

*   [Main page](https://wiki.archlinux.org/title/Main_page "Visit the main page [z]")
*   [Table of contents](https://wiki.archlinux.org/title/Table_of_contents)
*   [Getting involved](https://wiki.archlinux.org/title/Getting_involved "Various ways Archers can contribute to the community")
*   [Wiki news](https://wiki.archlinux.org/title/ArchWiki:News "The latest lowdown on the wiki")
*   [Random page](https://wiki.archlinux.org/title/Special:Random "Load a random page [x]")

 Interaction 

*   [Help](https://wiki.archlinux.org/title/Category:Help "Wiki navigation, reading, and editing help")
*   [Contributing](https://wiki.archlinux.org/title/ArchWiki:Contributing)
*   [Recent changes](https://wiki.archlinux.org/title/Special:RecentChanges "A list of recent changes in the wiki [r]")
*   [Recent talks](https://wiki.archlinux.org/index.php?title=Special:RecentChanges&namespace=all-discussions)
*   [New pages](https://wiki.archlinux.org/title/Special:NewPages)
*   [Statistics](https://wiki.archlinux.org/title/ArchWiki:Statistics)
*   [Requests](https://wiki.archlinux.org/title/ArchWiki_talk:Requests)

[**ArchWiki**](https://wiki.archlinux.org/title/Main_page)

[Search](https://wiki.archlinux.org/title/Special:Search "Search ArchWiki [f]")

Search

- [x] Appearance 

*   [Create account](https://wiki.archlinux.org/index.php?title=Special:CreateAccount&returnto=QEMU "You are encouraged to create an account and log in; however, it is not mandatory")
*   [Log in](https://wiki.archlinux.org/index.php?title=Special:UserLogin&returnto=QEMU "You are encouraged to log in; however, it is not mandatory [o]")

- [x] Personal tools 

*   [Create account](https://wiki.archlinux.org/index.php?title=Special:CreateAccount&returnto=QEMU "You are encouraged to create an account and log in; however, it is not mandatory")
*   [Log in](https://wiki.archlinux.org/index.php?title=Special:UserLogin&returnto=QEMU "You are encouraged to log in; however, it is not mandatory [o]")

## Contents

move to sidebar hide

*   [Beginning](https://wiki.archlinux.org/title/QEMU#)
*   [1 Installation](https://wiki.archlinux.org/title/QEMU#Installation)Toggle Installation subsection
    *   [1.1 QEMU variants](https://wiki.archlinux.org/title/QEMU#QEMU_variants)

    *   [1.2 Details on packages available in Arch Linux](https://wiki.archlinux.org/title/QEMU#Details_on_packages_available_in_Arch_Linux)

*   [2 Graphical front-ends for QEMU](https://wiki.archlinux.org/title/QEMU#Graphical_front-ends_for_QEMU)

*   [3 Creating a new virtualized system](https://wiki.archlinux.org/title/QEMU#Creating_a_new_virtualized_system)Toggle Creating a new virtualized system subsection
    *   [3.1 Creating a hard disk image](https://wiki.archlinux.org/title/QEMU#Creating_a_hard_disk_image)
        *   [3.1.1 Overlay storage images](https://wiki.archlinux.org/title/QEMU#Overlay_storage_images)

        *   [3.1.2 Resizing an image](https://wiki.archlinux.org/title/QEMU#Resizing_an_image)
            *   [3.1.2.1 Shrinking an image](https://wiki.archlinux.org/title/QEMU#Shrinking_an_image)

        *   [3.1.3 Converting an image](https://wiki.archlinux.org/title/QEMU#Converting_an_image)

    *   [3.2 Preparing the installation media](https://wiki.archlinux.org/title/QEMU#Preparing_the_installation_media)

    *   [3.3 Installing the operating system](https://wiki.archlinux.org/title/QEMU#Installing_the_operating_system)

    *   [3.4 Pre-made virtual machine images](https://wiki.archlinux.org/title/QEMU#Pre-made_virtual_machine_images)

*   [4 Running a virtualized system](https://wiki.archlinux.org/title/QEMU#Running_a_virtualized_system)Toggle Running a virtualized system subsection
    *   [4.1 Enabling KVM](https://wiki.archlinux.org/title/QEMU#Enabling_KVM)

    *   [4.2 Enabling IOMMU (Intel VT-d/AMD-Vi) support](https://wiki.archlinux.org/title/QEMU#Enabling_IOMMU_(Intel_VT-d/AMD-Vi)_support)

    *   [4.3 Booting in UEFI mode](https://wiki.archlinux.org/title/QEMU#Booting_in_UEFI_mode)
        *   [4.3.1 Enabling Secure Boot](https://wiki.archlinux.org/title/QEMU#Enabling_Secure_Boot)

    *   [4.4 Trusted Platform Module emulation](https://wiki.archlinux.org/title/QEMU#Trusted_Platform_Module_emulation)

*   [5 Communication between host and guest](https://wiki.archlinux.org/title/QEMU#Communication_between_host_and_guest)Toggle Communication between host and guest subsection
    *   [5.1 Network](https://wiki.archlinux.org/title/QEMU#Network)

    *   [5.2 QEMU's port forwarding](https://wiki.archlinux.org/title/QEMU#QEMU's_port_forwarding)

    *   [5.3 Accessing SSH via vsock](https://wiki.archlinux.org/title/QEMU#Accessing_SSH_via_vsock)

    *   [5.4 QEMU's built-in SMB server](https://wiki.archlinux.org/title/QEMU#QEMU's_built-in_SMB_server)
        *   [5.4.1 Share multiple directories](https://wiki.archlinux.org/title/QEMU#Share_multiple_directories)

    *   [5.5 Host file sharing with 9pfs VirtFS](https://wiki.archlinux.org/title/QEMU#Host_file_sharing_with_9pfs_VirtFS)

    *   [5.6 Host file sharing with virtiofsd](https://wiki.archlinux.org/title/QEMU#Host_file_sharing_with_virtiofsd)
        *   [5.6.1 Running virtiofsd as a regular user](https://wiki.archlinux.org/title/QEMU#Running_virtiofsd_as_a_regular_user)

        *   [5.6.2 Running virtiofsd as root](https://wiki.archlinux.org/title/QEMU#Running_virtiofsd_as_root)

        *   [5.6.3 Launching QEMU](https://wiki.archlinux.org/title/QEMU#Launching_QEMU)

        *   [5.6.4 Boot rootfs directly](https://wiki.archlinux.org/title/QEMU#Boot_rootfs_directly)

        *   [5.6.5 Using the share in a Linux guest](https://wiki.archlinux.org/title/QEMU#Using_the_share_in_a_Linux_guest)

        *   [5.6.6 Using the share in a Windows guest](https://wiki.archlinux.org/title/QEMU#Using_the_share_in_a_Windows_guest)

    *   [5.7 Mounting a partition of the guest on the host](https://wiki.archlinux.org/title/QEMU#Mounting_a_partition_of_the_guest_on_the_host)
        *   [5.7.1 Mounting a partition from a raw image](https://wiki.archlinux.org/title/QEMU#Mounting_a_partition_from_a_raw_image)
            *   [5.7.1.1 With manually specifying byte offset](https://wiki.archlinux.org/title/QEMU#With_manually_specifying_byte_offset)

            *   [5.7.1.2 With loop module autodetecting partitions](https://wiki.archlinux.org/title/QEMU#With_loop_module_autodetecting_partitions)

            *   [5.7.1.3 With kpartx](https://wiki.archlinux.org/title/QEMU#With_kpartx)

        *   [5.7.2 Mounting a partition from a qcow2 image](https://wiki.archlinux.org/title/QEMU#Mounting_a_partition_from_a_qcow2_image)

*   [6 Networking](https://wiki.archlinux.org/title/QEMU#Networking)Toggle Networking subsection
    *   [6.1 Link-level address caveat](https://wiki.archlinux.org/title/QEMU#Link-level_address_caveat)

    *   [6.2 User-mode networking](https://wiki.archlinux.org/title/QEMU#User-mode_networking)
        *   [6.2.1 SLIRP](https://wiki.archlinux.org/title/QEMU#SLIRP)

        *   [6.2.2 passt](https://wiki.archlinux.org/title/QEMU#passt)

    *   [6.3 Tap networking with QEMU](https://wiki.archlinux.org/title/QEMU#Tap_networking_with_QEMU)
        *   [6.3.1 Host-only networking](https://wiki.archlinux.org/title/QEMU#Host-only_networking)

        *   [6.3.2 Internal networking](https://wiki.archlinux.org/title/QEMU#Internal_networking)

        *   [6.3.3 Bridged networking using qemu-bridge-helper](https://wiki.archlinux.org/title/QEMU#Bridged_networking_using_qemu-bridge-helper)

        *   [6.3.4 Advanced network configuration](https://wiki.archlinux.org/title/QEMU#Advanced_network_configuration)

    *   [6.4 Shorthand configuration](https://wiki.archlinux.org/title/QEMU#Shorthand_configuration)

*   [7 Graphics card](https://wiki.archlinux.org/title/QEMU#Graphics_card)Toggle Graphics card subsection
    *   [7.1 std](https://wiki.archlinux.org/title/QEMU#std)

    *   [7.2 qxl](https://wiki.archlinux.org/title/QEMU#qxl)

    *   [7.3 vmware](https://wiki.archlinux.org/title/QEMU#vmware)

    *   [7.4 virtio](https://wiki.archlinux.org/title/QEMU#virtio)

    *   [7.5 cirrus](https://wiki.archlinux.org/title/QEMU#cirrus)

    *   [7.6 none](https://wiki.archlinux.org/title/QEMU#none)

*   [8 SPICE](https://wiki.archlinux.org/title/QEMU#SPICE)Toggle SPICE subsection
    *   [8.1 Enabling SPICE support on the host](https://wiki.archlinux.org/title/QEMU#Enabling_SPICE_support_on_the_host)

    *   [8.2 Connecting to the guest with a SPICE client](https://wiki.archlinux.org/title/QEMU#Connecting_to_the_guest_with_a_SPICE_client)
        *   [8.2.1 Manually running a SPICE client](https://wiki.archlinux.org/title/QEMU#Manually_running_a_SPICE_client)

        *   [8.2.2 Running a SPICE client with QEMU](https://wiki.archlinux.org/title/QEMU#Running_a_SPICE_client_with_QEMU)

    *   [8.3 Enabling SPICE support on the guest](https://wiki.archlinux.org/title/QEMU#Enabling_SPICE_support_on_the_guest)

    *   [8.4 Password authentication with SPICE](https://wiki.archlinux.org/title/QEMU#Password_authentication_with_SPICE)

    *   [8.5 TLS encrypted communication with SPICE](https://wiki.archlinux.org/title/QEMU#TLS_encrypted_communication_with_SPICE)

*   [9 VNC](https://wiki.archlinux.org/title/QEMU#VNC)Toggle VNC subsection
    *   [9.1 Basic password authentication](https://wiki.archlinux.org/title/QEMU#Basic_password_authentication)

*   [10 Audio](https://wiki.archlinux.org/title/QEMU#Audio)Toggle Audio subsection
    *   [10.1 Creating an audio backend](https://wiki.archlinux.org/title/QEMU#Creating_an_audio_backend)

    *   [10.2 Using the audio backend](https://wiki.archlinux.org/title/QEMU#Using_the_audio_backend)
        *   [10.2.1 Intel HD Audio](https://wiki.archlinux.org/title/QEMU#Intel_HD_Audio)

        *   [10.2.2 Intel 82801AA AC97](https://wiki.archlinux.org/title/QEMU#Intel_82801AA_AC97)

        *   [10.2.3 VirtIO sound](https://wiki.archlinux.org/title/QEMU#VirtIO_sound)

*   [11 Using virtio drivers](https://wiki.archlinux.org/title/QEMU#Using_virtio_drivers)Toggle Using virtio drivers subsection
    *   [11.1 Preparing an Arch Linux guest](https://wiki.archlinux.org/title/QEMU#Preparing_an_Arch_Linux_guest)
        *   [11.1.1 Memory ballooning](https://wiki.archlinux.org/title/QEMU#Memory_ballooning)

        *   [11.1.2 Using virtio pmem to bypass the guest's page cache](https://wiki.archlinux.org/title/QEMU#Using_virtio_pmem_to_bypass_the_guest's_page_cache)

    *   [11.2 Preparing a Windows guest](https://wiki.archlinux.org/title/QEMU#Preparing_a_Windows_guest)
        *   [11.2.1 Virtio drivers for Windows](https://wiki.archlinux.org/title/QEMU#Virtio_drivers_for_Windows)

        *   [11.2.2 Block device drivers](https://wiki.archlinux.org/title/QEMU#Block_device_drivers)
            *   [11.2.2.1 New Install of Windows](https://wiki.archlinux.org/title/QEMU#New_Install_of_Windows)

            *   [11.2.2.2 Change existing Windows virtual machine to use virtio](https://wiki.archlinux.org/title/QEMU#Change_existing_Windows_virtual_machine_to_use_virtio)

        *   [11.2.3 Network drivers](https://wiki.archlinux.org/title/QEMU#Network_drivers)

        *   [11.2.4 Balloon driver](https://wiki.archlinux.org/title/QEMU#Balloon_driver)

        *   [11.2.5 Using a virtiofsd share](https://wiki.archlinux.org/title/QEMU#Using_a_virtiofsd_share)

    *   [11.3 Preparing a FreeBSD guest](https://wiki.archlinux.org/title/QEMU#Preparing_a_FreeBSD_guest)

*   [12 QEMU monitor](https://wiki.archlinux.org/title/QEMU#QEMU_monitor)Toggle QEMU monitor subsection
    *   [12.1 Accessing the monitor console](https://wiki.archlinux.org/title/QEMU#Accessing_the_monitor_console)
        *   [12.1.1 Graphical view](https://wiki.archlinux.org/title/QEMU#Graphical_view)

        *   [12.1.2 Telnet](https://wiki.archlinux.org/title/QEMU#Telnet)

        *   [12.1.3 UNIX socket](https://wiki.archlinux.org/title/QEMU#UNIX_socket)

        *   [12.1.4 TCP](https://wiki.archlinux.org/title/QEMU#TCP)

        *   [12.1.5 Standard I/O](https://wiki.archlinux.org/title/QEMU#Standard_I/O)

    *   [12.2 Sending keyboard presses to the virtual machine using the monitor console](https://wiki.archlinux.org/title/QEMU#Sending_keyboard_presses_to_the_virtual_machine_using_the_monitor_console)

    *   [12.3 Creating and managing snapshots via the monitor console](https://wiki.archlinux.org/title/QEMU#Creating_and_managing_snapshots_via_the_monitor_console)

    *   [12.4 Running the virtual machine in immutable mode](https://wiki.archlinux.org/title/QEMU#Running_the_virtual_machine_in_immutable_mode)

    *   [12.5 Pause and power options via the monitor console](https://wiki.archlinux.org/title/QEMU#Pause_and_power_options_via_the_monitor_console)

    *   [12.6 Taking screenshots of the virtual machine](https://wiki.archlinux.org/title/QEMU#Taking_screenshots_of_the_virtual_machine)

*   [13 QEMU machine protocol](https://wiki.archlinux.org/title/QEMU#QEMU_machine_protocol)Toggle QEMU machine protocol subsection
    *   [13.1 Start QMP](https://wiki.archlinux.org/title/QEMU#Start_QMP)

    *   [13.2 Live merging of child image into parent image](https://wiki.archlinux.org/title/QEMU#Live_merging_of_child_image_into_parent_image)

    *   [13.3 Live creation of a new snapshot](https://wiki.archlinux.org/title/QEMU#Live_creation_of_a_new_snapshot)

*   [14 Tips and tricks](https://wiki.archlinux.org/title/QEMU#Tips_and_tricks)Toggle Tips and tricks subsection
    *   [14.1 Improve virtual machine performance](https://wiki.archlinux.org/title/QEMU#Improve_virtual_machine_performance)

    *   [14.2 Using any real partition as the single primary partition of a hard disk image](https://wiki.archlinux.org/title/QEMU#Using_any_real_partition_as_the_single_primary_partition_of_a_hard_disk_image)
        *   [14.2.1 Specifying kernel and initramfs manually](https://wiki.archlinux.org/title/QEMU#Specifying_kernel_and_initramfs_manually)

        *   [14.2.2 Simulating a virtual disk with MBR](https://wiki.archlinux.org/title/QEMU#Simulating_a_virtual_disk_with_MBR)
            *   [14.2.2.1 Using the device-mapper](https://wiki.archlinux.org/title/QEMU#Using_the_device-mapper)

            *   [14.2.2.2 Using a linear RAID](https://wiki.archlinux.org/title/QEMU#Using_a_linear_RAID)

            *   [14.2.2.3 Using a Network Block Device](https://wiki.archlinux.org/title/QEMU#Using_a_Network_Block_Device)

    *   [14.3 Starting QEMU virtual machines on boot](https://wiki.archlinux.org/title/QEMU#Starting_QEMU_virtual_machines_on_boot)
        *   [14.3.1 With libvirt](https://wiki.archlinux.org/title/QEMU#With_libvirt)

        *   [14.3.2 With systemd service](https://wiki.archlinux.org/title/QEMU#With_systemd_service)

    *   [14.4 Mouse integration](https://wiki.archlinux.org/title/QEMU#Mouse_integration)

    *   [14.5 Pass-through host USB device](https://wiki.archlinux.org/title/QEMU#Pass-through_host_USB_device)

    *   [14.6 USB redirection with SPICE](https://wiki.archlinux.org/title/QEMU#USB_redirection_with_SPICE)
        *   [14.6.1 Automatic USB forwarding with udev](https://wiki.archlinux.org/title/QEMU#Automatic_USB_forwarding_with_udev)

    *   [14.7 Enabling KSM](https://wiki.archlinux.org/title/QEMU#Enabling_KSM)

    *   [14.8 Multi-monitor support](https://wiki.archlinux.org/title/QEMU#Multi-monitor_support)

    *   [14.9 Custom display resolution](https://wiki.archlinux.org/title/QEMU#Custom_display_resolution)

    *   [14.10 Copy and paste](https://wiki.archlinux.org/title/QEMU#Copy_and_paste)
        *   [14.10.1 SPICE](https://wiki.archlinux.org/title/QEMU#SPICE_2)

        *   [14.10.2 qemu-vdagent](https://wiki.archlinux.org/title/QEMU#qemu-vdagent)

    *   [14.11 Windows-specific notes](https://wiki.archlinux.org/title/QEMU#Windows-specific_notes)
        *   [14.11.1 Fast startup](https://wiki.archlinux.org/title/QEMU#Fast_startup)

        *   [14.11.2 Remote Desktop Protocol](https://wiki.archlinux.org/title/QEMU#Remote_Desktop_Protocol)

        *   [14.11.3 Time standard](https://wiki.archlinux.org/title/QEMU#Time_standard)

    *   [14.12 Clone Linux system installed on physical equipment](https://wiki.archlinux.org/title/QEMU#Clone_Linux_system_installed_on_physical_equipment)

    *   [14.13 Chrooting into arm/arm64 environment from x86_64](https://wiki.archlinux.org/title/QEMU#Chrooting_into_arm/arm64_environment_from_x86_64)
        *   [14.13.1 sudo in chroot](https://wiki.archlinux.org/title/QEMU#sudo_in_chroot)

    *   [14.14 Not grabbing mouse input](https://wiki.archlinux.org/title/QEMU#Not_grabbing_mouse_input)

*   [15 Troubleshooting](https://wiki.archlinux.org/title/QEMU#Troubleshooting)

*   [16 See also](https://wiki.archlinux.org/title/QEMU#See_also)

- [x] Toggle the table of contents 

