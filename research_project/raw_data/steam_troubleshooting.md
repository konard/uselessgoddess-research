Title: Steam/Troubleshooting - ArchWiki

URL Source: https://wiki.archlinux.org/title/Steam/Troubleshooting

Published Time: Mon, 30 Mar 2026 00:46:20 GMT

Markdown Content:
# Steam/Troubleshooting - ArchWiki

[](https://archlinux.org/)

*   [Home](https://archlinux.org/)
*   [Packages](https://archlinux.org/packages/)
*   [Forums](https://bbs.archlinux.org/)
*   [Wiki](https://wiki.archlinux.org/)
*   [GitLab](https://gitlab.archlinux.org/archlinux)
*   [Security](https://security.archlinux.org/)
*   [AUR](https://aur.archlinux.org/)
*   [Download](https://archlinux.org/download/)

[Jump to content](https://wiki.archlinux.org/title/Steam/Troubleshooting#bodyContent)

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

*   [Create account](https://wiki.archlinux.org/index.php?title=Special:CreateAccount&returnto=Steam%2FTroubleshooting "You are encouraged to create an account and log in; however, it is not mandatory")
*   [Log in](https://wiki.archlinux.org/index.php?title=Special:UserLogin&returnto=Steam%2FTroubleshooting "You are encouraged to log in; however, it is not mandatory [o]")

- [x] Personal tools 

*   [Create account](https://wiki.archlinux.org/index.php?title=Special:CreateAccount&returnto=Steam%2FTroubleshooting "You are encouraged to create an account and log in; however, it is not mandatory")
*   [Log in](https://wiki.archlinux.org/index.php?title=Special:UserLogin&returnto=Steam%2FTroubleshooting "You are encouraged to log in; however, it is not mandatory [o]")

## Contents

move to sidebar hide

*   [Beginning](https://wiki.archlinux.org/title/Steam/Troubleshooting#)
*   [1 Steam runtime](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_runtime)Toggle Steam runtime subsection
    *   [1.1 Steam native runtime](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_native_runtime)

*   [2 Debugging shared libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Debugging_shared_libraries)Toggle Debugging shared libraries subsection
    *   [2.1 Finding missing game libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Finding_missing_game_libraries)

    *   [2.2 Finding missing runtime libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Finding_missing_runtime_libraries)

*   [3 Debugging Steam](https://wiki.archlinux.org/title/Steam/Troubleshooting#Debugging_Steam)

*   [4 Runtime issues](https://wiki.archlinux.org/title/Steam/Troubleshooting#Runtime_issues)Toggle Runtime issues subsection
    *   [4.1'GLBCXX_3.X.XX' not found when using Bumblebee](https://wiki.archlinux.org/title/Steam/Troubleshooting#'GLBCXX_3.X.XX'_not_found_when_using_Bumblebee)

    *   [4.2 Steam>Warning: failed to init SDL thread priority manager: SDL not found](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam%3EWarning:_failed_to_init_SDL_thread_priority_manager:_SDL_not_found)

    *   [4.3 Game crashes immediately](https://wiki.archlinux.org/title/Steam/Troubleshooting#Game_crashes_immediately)

    *   [4.4 Game and Steam crashes after game start](https://wiki.archlinux.org/title/Steam/Troubleshooting#Game_and_Steam_crashes_after_game_start)

    *   [4.5 Some games freeze at start when in focus](https://wiki.archlinux.org/title/Steam/Troubleshooting#Some_games_freeze_at_start_when_in_focus)

    *   [4.6 Version `CURL_OPENSSL_3` not found](https://wiki.archlinux.org/title/Steam/Troubleshooting#Version_%60CURL_OPENSSL_3%60_not_found)

    *   [4.7 Steam webview/game browser not working in native runtime (Black screen)](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_webview/game_browser_not_working_in_native_runtime_(Black_screen))

    *   [4.8 Steam: An X Error occurred](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam:_An_X_Error_occurred)

    *   [4.9 Steam: Compatibility tool configuration failed](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam:_Compatibility_tool_configuration_failed)

    *   [4.10 Game starts but closes immediately with custom kernel](https://wiki.archlinux.org/title/Steam/Troubleshooting#Game_starts_but_closes_immediately_with_custom_kernel)

    *   [4.11 Steam Library won't start](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_Library_won't_start)

*   [5 Graphical issues](https://wiki.archlinux.org/title/Steam/Troubleshooting#Graphical_issues)Toggle Graphical issues subsection
    *   [5.1 Black main screen on Intel iGPUs](https://wiki.archlinux.org/title/Steam/Troubleshooting#Black_main_screen_on_Intel_iGPUs)

    *   [5.2 Blurry text and graphics with Xwayland and HiDPI](https://wiki.archlinux.org/title/Steam/Troubleshooting#Blurry_text_and_graphics_with_Xwayland_and_HiDPI)

    *   [5.3 Steam flicker/blink with black screen not loading Store/Library or other pages](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_flicker/blink_with_black_screen_not_loading_Store/Library_or_other_pages)
        *   [5.3.1 Fix by editing desktop entry](https://wiki.archlinux.org/title/Steam/Troubleshooting#Fix_by_editing_desktop_entry)

        *   [5.3.2 Bypass desktop entry](https://wiki.archlinux.org/title/Steam/Troubleshooting#Bypass_desktop_entry)

*   [6 Audio issues](https://wiki.archlinux.org/title/Steam/Troubleshooting#Audio_issues)Toggle Audio issues subsection
    *   [6.1 Configure PulseAudio](https://wiki.archlinux.org/title/Steam/Troubleshooting#Configure_PulseAudio)

    *   [6.2 No audio or 756 Segmentation fault](https://wiki.archlinux.org/title/Steam/Troubleshooting#No_audio_or_756_Segmentation_fault)

    *   [6.3 FMOD sound engine](https://wiki.archlinux.org/title/Steam/Troubleshooting#FMOD_sound_engine)

    *   [6.4 PulseAudio & OpenAL: Audio streams cannot be moved between devices](https://wiki.archlinux.org/title/Steam/Troubleshooting#PulseAudio_&_OpenAL:_Audio_streams_cannot_be_moved_between_devices)

    *   [6.5 Cracking Microphone in Steam Voice and Games](https://wiki.archlinux.org/title/Steam/Troubleshooting#Cracking_Microphone_in_Steam_Voice_and_Games)

*   [7 Steam client issues](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_client_issues)Toggle Steam client issues subsection
    *   [7.1 Cannot browse filesystem to add a library folder or library folder appears as empty](https://wiki.archlinux.org/title/Steam/Troubleshooting#Cannot_browse_filesystem_to_add_a_library_folder_or_library_folder_appears_as_empty)

    *   [7.2 Cannot add library folder because of missing execute permissions](https://wiki.archlinux.org/title/Steam/Troubleshooting#Cannot_add_library_folder_because_of_missing_execute_permissions)

    *   [7.3 Unusually slow download speed](https://wiki.archlinux.org/title/Steam/Troubleshooting#Unusually_slow_download_speed)

    *   [7.4"Needs to be online" error](https://wiki.archlinux.org/title/Steam/Troubleshooting#%22Needs_to_be_online%22_error)

    *   [7.5 Steam forgets password](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_forgets_password)

    *   [7.6 Preventing crash memory dumps](https://wiki.archlinux.org/title/Steam/Troubleshooting#Preventing_crash_memory_dumps)

    *   [7.7 Steam license problem with playing videos](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_license_problem_with_playing_videos)

    *   [7.8 No context menu for joining/inviting friends](https://wiki.archlinux.org/title/Steam/Troubleshooting#No_context_menu_for_joining/inviting_friends)

    *   [7.9 Slow and unresponsive user interface](https://wiki.archlinux.org/title/Steam/Troubleshooting#Slow_and_unresponsive_user_interface)

    *   [7.10 Steam fails to start correctly](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_fails_to_start_correctly)

    *   [7.11 Missing taskbar menu](https://wiki.archlinux.org/title/Steam/Troubleshooting#Missing_taskbar_menu)

    *   [7.12"Your browser does not support the minimum set of features required to watch this broadcast" error](https://wiki.archlinux.org/title/Steam/Troubleshooting#%22Your_browser_does_not_support_the_minimum_set_of_features_required_to_watch_this_broadcast%22_error)

    *   [7.13 Using system titlebar and frame](https://wiki.archlinux.org/title/Steam/Troubleshooting#Using_system_titlebar_and_frame)

    *   [7.14 More selective DPMS inhibition](https://wiki.archlinux.org/title/Steam/Troubleshooting#More_selective_DPMS_inhibition)

    *   [7.15 Enabling fractional scaling](https://wiki.archlinux.org/title/Steam/Troubleshooting#Enabling_fractional_scaling)

    *   [7.16 Steam Beta crashes](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_Beta_crashes)

    *   [7.17 Cannot access store page (client displays error -105 or -102)](https://wiki.archlinux.org/title/Steam/Troubleshooting#Cannot_access_store_page_(client_displays_error_-105_or_-102))

    *   [7.18 Steam client restarts while a game is running](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_client_restarts_while_a_game_is_running)

*   [8 Steam Remote Play issues](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_Remote_Play_issues)Toggle Steam Remote Play issues subsection
    *   [8.1 Remote Play does not work from Arch Linux host to Arch Linux guest](https://wiki.archlinux.org/title/Steam/Troubleshooting#Remote_Play_does_not_work_from_Arch_Linux_host_to_Arch_Linux_guest)

    *   [8.2 Hardware decoding not available](https://wiki.archlinux.org/title/Steam/Troubleshooting#Hardware_decoding_not_available)

    *   [8.3 Big Picture Mode minimizes itself after losing focus](https://wiki.archlinux.org/title/Steam/Troubleshooting#Big_Picture_Mode_minimizes_itself_after_losing_focus)

*   [9 Other issues](https://wiki.archlinux.org/title/Steam/Troubleshooting#Other_issues)Toggle Other issues subsection
    *   [9.1 Steam Library in NTFS partition](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_Library_in_NTFS_partition)

    *   [9.2 Wrong ELF class](https://wiki.archlinux.org/title/Steam/Troubleshooting#Wrong_ELF_class)

    *   [9.3 Multiple monitors setup](https://wiki.archlinux.org/title/Steam/Troubleshooting#Multiple_monitors_setup)

    *   [9.4 Text is corrupt or missing](https://wiki.archlinux.org/title/Steam/Troubleshooting#Text_is_corrupt_or_missing)

    *   [9.5 SetLocale('en_US.UTF-8') fails at game startup or typing non-ASCII characters does not work in the Steam client](https://wiki.archlinux.org/title/Steam/Troubleshooting#SetLocale('en_US.UTF-8')_fails_at_game_startup_or_typing_non-ASCII_characters_does_not_work_in_the_Steam_client)

    *   [9.6 Missing libc](https://wiki.archlinux.org/title/Steam/Troubleshooting#Missing_libc)

    *   [9.7 Games do not launch on older Intel hardware](https://wiki.archlinux.org/title/Steam/Troubleshooting#Games_do_not_launch_on_older_Intel_hardware)

    *   [9.8 Mesa: Game does not launch, complaining about OpenGL version supported by the card](https://wiki.archlinux.org/title/Steam/Troubleshooting#Mesa:_Game_does_not_launch,_complaining_about_OpenGL_version_supported_by_the_card)

    *   [9.9 2K games do not run on XFS partitions](https://wiki.archlinux.org/title/Steam/Troubleshooting#2K_games_do_not_run_on_XFS_partitions)

    *   [9.10 Steam controller not being detected correctly](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_controller_not_being_detected_correctly)

    *   [9.11 Steam controller makes a game crash](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_controller_makes_a_game_crash)

    *   [9.12 Steam hangs on "Installing breakpad exception handler..."](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_hangs_on_%22Installing_breakpad_exception_handler...%22)

    *   [9.13 Killing standalone compositors when launching games](https://wiki.archlinux.org/title/Steam/Troubleshooting#Killing_standalone_compositors_when_launching_games)

    *   [9.14 Symbol lookup error using DRI3](https://wiki.archlinux.org/title/Steam/Troubleshooting#Symbol_lookup_error_using_DRI3)

    *   [9.15 Launching games on NVIDIA Optimus laptops](https://wiki.archlinux.org/title/Steam/Troubleshooting#Launching_games_on_NVIDIA_Optimus_laptops)

    *   [9.16 HiDPI](https://wiki.archlinux.org/title/Steam/Troubleshooting#HiDPI)

    *   [9.17 Protocol support under KDE Plasma](https://wiki.archlinux.org/title/Steam/Troubleshooting#Protocol_support_under_KDE_Plasma)

    *   [9.18 The game crashes when using Steam Linux Runtime - Soldier](https://wiki.archlinux.org/title/Steam/Troubleshooting#The_game_crashes_when_using_Steam_Linux_Runtime_-_Soldier)

    *   [9.19 Games running with Proton 5.13+ have no Internet connectivity](https://wiki.archlinux.org/title/Steam/Troubleshooting#Games_running_with_Proton_5.13+_have_no_Internet_connectivity)

    *   [9.20"could not determine 32/64 bit of java"](https://wiki.archlinux.org/title/Steam/Troubleshooting#%22could_not_determine_32/64_bit_of_java%22)

    *   [9.21 Stuttering with Vulkan](https://wiki.archlinux.org/title/Steam/Troubleshooting#Stuttering_with_Vulkan)

    *   [9.22 Force OpenGL emulation](https://wiki.archlinux.org/title/Steam/Troubleshooting#Force_OpenGL_emulation)

    *   [9.23 File picker does not see anything but Steam library](https://wiki.archlinux.org/title/Steam/Troubleshooting#File_picker_does_not_see_anything_but_Steam_library)

    *   [9.24 DirectX errors on hybrid graphics](https://wiki.archlinux.org/title/Steam/Troubleshooting#DirectX_errors_on_hybrid_graphics)

    *   [9.25 No Internet Connection when downloading](https://wiki.archlinux.org/title/Steam/Troubleshooting#No_Internet_Connection_when_downloading)

    *   [9.26 Poor performance or stuttering after launching Steam](https://wiki.archlinux.org/title/Steam/Troubleshooting#Poor_performance_or_stuttering_after_launching_Steam)

    *   [9.27 Very long startup and slow user interface response](https://wiki.archlinux.org/title/Steam/Troubleshooting#Very_long_startup_and_slow_user_interface_response)

*   [10 See also](https://wiki.archlinux.org/title/Steam/Troubleshooting#See_also)

- [x] Toggle the table of contents 

# Steam/Troubleshooting

- [x] 4 languages 

*   [Magyar](https://wiki.archlinux.org/title/Steam_(Magyar)/Troubleshooting_(Magyar) "Steam (Magyar)/Troubleshooting – magyar")
*   [日本語](https://wiki.archlinux.jp/index.php/Steam/%E3%83%88%E3%83%A9%E3%83%96%E3%83%AB%E3%82%B7%E3%83%A5%E3%83%BC%E3%83%86%E3%82%A3%E3%83%B3%E3%82%B0 "Steam/トラブルシューティング – 日本語")
*   [Русский](https://wiki.archlinux.org/title/Steam_(%D0%A0%D1%83%D1%81%D1%81%D0%BA%D0%B8%D0%B9)/Troubleshooting_(%D0%A0%D1%83%D1%81%D1%81%D0%BA%D0%B8%D0%B9) "Steam (Русский)/Troubleshooting – русский")
*   [中文（简体）](https://wiki.archlinuxcn.org/wiki/Steam/%E7%96%91%E9%9A%BE%E8%A7%A3%E7%AD%94 "Steam/疑难解答 – 中文（简体）")

*   [Page](https://wiki.archlinux.org/title/Steam/Troubleshooting "View the content page [c]")
*   [Discussion](https://wiki.archlinux.org/title/Talk:Steam/Troubleshooting "Discussion about the content page [t]")

- [x] English 

*   [Read](https://wiki.archlinux.org/title/Steam/Troubleshooting)
*   [View source](https://wiki.archlinux.org/index.php?title=Steam/Troubleshooting&action=edit "This page is protected.
You can view its source [e]")
*   [View history](https://wiki.archlinux.org/index.php?title=Steam/Troubleshooting&action=history "Past revisions of this page [h]")

- [x] Tools 

Tools

move to sidebar hide

 Actions 

*   [Read](https://wiki.archlinux.org/title/Steam/Troubleshooting)
*   [View source](https://wiki.archlinux.org/index.php?title=Steam/Troubleshooting&action=edit)
*   [View history](https://wiki.archlinux.org/index.php?title=Steam/Troubleshooting&action=history)

 General 

*   [What links here](https://wiki.archlinux.org/title/Special:WhatLinksHere/Steam/Troubleshooting "A list of all wiki pages that link here [j]")
*   [Related changes](https://wiki.archlinux.org/title/Special:RecentChangesLinked/Steam/Troubleshooting "Recent changes in pages linked from this page [k]")
*   [Printable version](javascript:print(); "Printable version of this page [p]")
*   [Permanent link](https://wiki.archlinux.org/index.php?title=Steam/Troubleshooting&oldid=868814 "Permanent link to this revision of this page")
*   [Page information](https://wiki.archlinux.org/index.php?title=Steam/Troubleshooting&action=info "More information about this page")

Appearance

move to sidebar hide

From ArchWiki

<[Steam](https://wiki.archlinux.org/title/Steam "Steam")

1.   Make sure that you have followed [Steam#Installation](https://wiki.archlinux.org/title/Steam#Installation "Steam").
2.   If the Steam client / a game is not starting and/or you have error message about a library, read [#Steam runtime](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_runtime) and see [#Debugging shared libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Debugging_shared_libraries).
3.   If the issue is related to networking, make sure that you have forwarded the [required ports for Steam](https://help.steampowered.com/en/faqs/view/2EA8-4D75-DA21-31EB).
4.   If the issue is about a game, consult [Steam/Game-specific troubleshooting](https://wiki.archlinux.org/title/Steam/Game-specific_troubleshooting "Steam/Game-specific troubleshooting").

## Steam runtime

Steam for Linux ships with its own set of libraries called the [Steam runtime](https://github.com/ValveSoftware/steam-runtime). By default Steam launches all Steam Applications within the runtime environment. The Steam runtime is located at `~/.steam/root/ubuntu12_32/steam-runtime/`.

If you mix the Steam runtime libraries with system libraries you will run into binary incompatibility issues, see [steam-for-linux issue #4768](https://github.com/ValveSoftware/steam-for-linux/issues/4768). Binary incompatibility can lead to the Steam client and games not starting (manifesting as a crash, as hanging or silently returning), audio issues and various other problems.

The [steam](https://archlinux.org/packages/?name=steam) package offers two ways to launch Steam:

*   `/usr/bin/steam` (alias `steam`), which overrides runtime libraries known to cause problems via the `LD_PRELOAD`[environment variable](https://wiki.archlinux.org/title/Environment_variable "Environment variable") (see [ld.so(8)](https://man.archlinux.org/man/ld.so.8)).
*   `/usr/lib/steam/steam`, the default Steam launch script

As the Steam runtime libraries are older they can lack newer features, e.g. the OpenAL version of the Steam runtime lacks [HRTF](https://wiki.archlinux.org/title/Gaming#Binaural_audio_with_OpenAL "Gaming") and surround71 support.

### Steam native runtime

**Warning** Using the Steam native runtime is not recommended as it might break some games due to binary incompatibility and it might miss some libraries present in the Steam runtime.

The [steam-native-runtime](https://aur.archlinux.org/packages/steam-native-runtime/)AUR package depends on over 130 packages to pose a native replacement of the Steam runtime, some games may however still require additional packages.

This package provides the `steam-native` script, which launches Steam with the `STEAM_RUNTIME=0` environment variable and `-compat-force-slr off` flag, making it ignore its runtime and only use system libraries.

**Note** This will only apply to games that use the Steam Linux Runtime 1 and Steam itself, if developers choose to use Steam Linux Runtime 3 or newer, take for example Barony, the only way to escape that will be to either launch the game not from Steam or modify the launch arguments to something like `./<game_executable>; exit; %command%`, which launches the game in system environment since runtimes are only applied for `%command%`.

You can also use the Steam native runtime without [steam-native-runtime](https://aur.archlinux.org/packages/steam-native-runtime/)AUR by manually installing just the packages you need. See [#Finding missing runtime libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Finding_missing_runtime_libraries).

## Debugging shared libraries

To see the shared libraries required by a program or a shared library run the `ldd` command on it, see [ldd(1)](https://man.archlinux.org/man/ldd.1). The `LD_LIBRARY_PATH` and `LD_PRELOAD`[environment variables](https://wiki.archlinux.org/title/Environment_variables "Environment variables") can alter which shared libraries are loaded, see [ld.so(8)](https://man.archlinux.org/man/ld.so.8). To correctly debug a program or shared library it is therefore important that these environment variables in your debug environment match the environment you wish to debug.

If you figure out a missing library you can use [pacman](https://wiki.archlinux.org/title/Pacman "Pacman") or [pkgfile](https://wiki.archlinux.org/title/Pkgfile "Pkgfile") to search for packages that contain the missing library.

### Finding missing game libraries

If a game fails to start, a possible reason is that it is missing required libraries. You can find out what libraries it requests by running `ldd game_executable`. `game_executable` is likely located somewhere in `~/.steam/root/steamapps/common/`. Please note that most of these "missing" libraries are actually already included with Steam, and do not need to be installed globally.

### Finding missing runtime libraries

If individual games or Steam itself is failing to launch when using `steam-native` you are probably missing libraries. To find the required libraries run:

$ cd ~/.steam/root/ubuntu12_32
$ file * | grep ELF | cut -d: -f1 | LD_LIBRARY_PATH=. xargs ldd | grep 'not found' | sort | uniq

Alternatively, run Steam with `steam` and use the following command to see which non-system libraries Steam is using (not all of these are part of the Steam runtime):

$ for i in $(pgrep steam); do sed '/\.local/!d;s/.*  //g' /proc/$i/maps; done | sort | uniq

## Debugging Steam

![Image 1](https://wiki.archlinux.org/images/4/4e/View-refresh-red.svg)**This article or section is out of date.**

**Reason:** Steam no longer redirects stdout and stderr to `/tmp/dumps/USER_stdout.txt` by default. See: [steam-for-linux issue 7114](https://github.com/ValveSoftware/steam-for-linux/issues/7114) A similar effect can be achieved by starting steam with `steam 2>&1` (Discuss in [Talk:Steam/Troubleshooting](https://wiki.archlinux.org/title/Talk:Steam/Troubleshooting))

The Steam launcher redirects its stdout and stderr to `/tmp/dumps/USER_stdout.txt`. This means you do not have to run Steam from the command-line to see that output.

It is possible to debug Steam to gain more information which could be useful to find out why something does not work.

You can set `DEBUGGER` environment variable with one of `gdb`, `cgdb`, `valgrind`, `callgrind`, `strace` and then start `steam`.

For example with [gdb](https://archlinux.org/packages/?name=gdb)

$ DEBUGGER=gdb steam

`gdb` will open, then type `run` which will start `steam` and once crash happens you can type `backtrace` to see call stack.

## Runtime issues

### 'GLBCXX_3.X.XX' not found when using Bumblebee

This error is likely caused because Steam packages its own out of date `libstdc++.so.6`. See [#Finding missing runtime libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Finding_missing_runtime_libraries) about working around the bad library. See also [steam-for-linux issue 3773](https://github.com/ValveSoftware/steam-for-linux/issues/3773).

### Steam>Warning: failed to init SDL thread priority manager: SDL not found

Solution: [install](https://wiki.archlinux.org/title/Install "Install") the [lib32-sdl2](https://aur.archlinux.org/packages/lib32-sdl2/)AUR package.

### Game crashes immediately

This is likely due to [#Steam runtime](https://wiki.archlinux.org/title/Steam/Troubleshooting#Steam_runtime) issues, see [#Debugging shared libraries](https://wiki.archlinux.org/title/Steam/Troubleshooting#Debugging_shared_libraries).

Disabling the in-game Steam Overlay in the game properties might help.

And finally, if those do not work, you should check Steam's output for any error from the game. You may encounter the following:

*   `munmap_chunk(): invalid pointer`
*   `free(): invalid pointer`

In these cases, try replacing the `libsteam_api.so` file from the problematic game with one of a game that works. This error usually happens for games that were not updated recently when Steam runtime is disabled. This error has been encountered with AYIM, Bastion and Monaco.

If the game crashes with

`terminate called after throwing an instance of 'dxvk::DxvkError'`

it's likely that conflicting versions of Vulkan are [installed](https://www.reddit.com/r/archlinux/comments/s1vjjg/comment/hsb10ac/?context=3). [lib32-vulkan-intel](https://archlinux.org/packages/?name=lib32-vulkan-intel) and NVIDIA Vulkan drivers are mutually exclusive. This is solved by uninstalling the unneeded driver. To obtain information about the chipset vendor one can run:

# lshw -C display | grep vendor

To get a list of installed packages

# pacman -Qs vulkan

### Game and Steam crashes after game start

If the following error is output:

failed to dlopen engine.so error=/home/_GAMEPATH_/bin/libgcc_s.so.1: version `GCC_7.0.0' not found (required by /usr/lib32/libopenal.so.1)

moving the incompatible lib can be a workaround.

mv .local/share/Steam/steamapps/common/_GAME_/bin/libgcc_s.so.1 .local/share/Steam/steamapps/common/_GAME_/bin/libgcc_s.so.1.b

### Some games freeze at start when in focus

A combination of using `ForceFullCompositionPipeline`, specific Proton versions and NVIDIA driver version 535 is known [to freeze some games](https://github.com/ValveSoftware/Proton/issues/6869)[using DXVK/Vulkan](https://github.com/doitsujin/dxvk/issues/3670) at launch under Xorg. Using Alt+Tab allows bringing Steam in focus, and the game seems to run properly in the background. Solution: disable `ForceFullCompositionPipeline` or downgrade NVIDIA drivers.

### Version `CURL_OPENSSL_3` not found

This is because [curl](https://archlinux.org/packages/?name=curl) alone is not compatible with previous versions. You need to install the compatibility libraries:

One of the following messages may show up:

# Nuclear Throne
./nuclearthrone: /usr/lib32/libcurl.so.4: version `CURL_OPENSSL_3' not found (required by ./nuclearthrone)

# Devil Daggers
./devildaggers: /usr/lib/libcurl.so.4: version `CURL_OPENSSL_3' not found (required by ./devildaggers)

You need to install either [libcurl-compat](https://archlinux.org/packages/?name=libcurl-compat) or [lib32-libcurl-compat](https://archlinux.org/packages/?name=lib32-libcurl-compat) and link the compatibility library manually:

# Nuclear Throne
$ ln -s /usr/lib32/libcurl-compat.so.4.4.0 "_LIBRARY_/steamapps/common/Nuclear Throne/lib/libcurl.so.4"

# Devil Daggers
$ ln -s /usr/lib/libcurl-compat.so.4.4.0 _LIBRARY_/steamapps/common/devildaggers/lib64/libcurl.so.4

### Steam webview/game browser not working in native runtime (Black screen)

Since the new Steam Friends UI update, the client webview is not working correctly with the native-runtime.

./steamwebhelper: error while loading shared libraries: libpcre.so.3: cannot open shared object file: No such file or directory

It can be solved preloading glib libraries; Those do not require libpcre and selinux to work.

$ LD_PRELOAD="/usr/lib/libgio-2.0.so.0 /usr/lib/libglib-2.0.so.0" steam-native

Alternatively, you may create a symbolic link to the native Arch libpcre library.

# ln -s /usr/lib/libpcre.so /usr/lib64/libpcre.so.3

Since update from around 3/3/2022, there are some reports of black screen still persisting after applying above workaround.

The workaround for now is to run Steam with the `-no-cef-sandbox` option. More information can be found in Github Steam-For-Linux repository Issue [#8451](https://github.com/ValveSoftware/steam-for-linux/issues/8451) and [#8420](https://github.com/ValveSoftware/steam-for-linux/issues/8420).

### Steam: An X Error occurred

When using an NVIDIA GPU and proprietary drivers, Steam may fail to start and (if run from the terminal) produce errors of the form:

Steam: An X Error occurred
X Error of failed request:  GLXBadContext
Major opcode of failed request:  151
Serial number of failed request:  51
xerror_handler: X failed, continuing

Ensure the _lib32-_[NVIDIA](https://wiki.archlinux.org/title/NVIDIA "NVIDIA") driver for your card is installed, and matches the main package version with:

# pacman -Qs nvidia

You may need to change which [mirrors](https://wiki.archlinux.org/title/Mirrors "Mirrors") you are using to install the drivers if they do not match.

If you are using AMD, have enabled 10-bit color depth, and are having this problem. You will likely need to disable 10-bit color depth.

Another issue that causes this error message can be [solved by removing the config.vdf file](https://github.com/ValveSoftware/steam-for-linux/issues/4340#issuecomment-258593713):

$ rm ~/.local/share/Steam/config/config.vdf

### Steam: Compatibility tool configuration failed

If you are trying to run a native game using Proton but get a Steam compatibility tool error immediately after starting the game, you might have to reinstall the runtime.

1.   Navigate to your Steam library.
2.   In the dropdown above your game list check the _Tools_ option to make them visible.
3.   Search for _Proton_, right click on each installed tool, visit _Properties_, open the _Local files_ tab and click _Verify integrity of tool files_ for each entry.
4.   Search for _Steam Linux Runtime_ and repeat the same procedure. If none are available, install the latest _Steam Linux Runtime - Soldier_.

### Game starts but closes immediately with custom kernel

Make sure that you have enabled _User namespace_ in _General setup -> Namespaces support_.

### Steam Library won't start

Opening the Steam library either displays nothing, or a brief splash, but no window appears. Running `/usr/bin/steam` in a terminal window gives this error:

Assertion 'device' failed at src/libsystemd/sd-device/device-private.c:103, function device_get_tags_generation(). Aborting.

Bugs reports are filed: [#79006](https://bugs.archlinux.org/task/79006)

See also discussion at: [Steam failing to launch since systemd 253.5-2 update](https://bbs.archlinux.org/viewtopic.php?id=287033)

A workaround is to install [lib32-libnm](https://archlinux.org/packages/?name=lib32-libnm).

## Graphical issues

### Black main screen on Intel iGPUs

On some systems that use Intel integrated graphics running Steam on [Wayland](https://wiki.archlinux.org/title/Wayland "Wayland") will make only the webviews not render, with functional dropdowns and other menus. Going to _Steam -> Settings -> Interface_, then disabling _Enable GPU accelerated rendering in web views_ might fix the problem.

### Blurry text and graphics with Xwayland and HiDPI

When Steam runs as an [Xwayland](https://wiki.archlinux.org/title/Xwayland "Xwayland") client under a compositor that uses [HiDPI](https://wiki.archlinux.org/title/HiDPI "HiDPI") scaling, you may find that Steam and games are rendered at half resolution and then upscaled to fit the HiDPI screen. This results in blurry graphics.

One option is to run Steam under a nested [gamescope](https://wiki.archlinux.org/title/Gamescope "Gamescope") compositor. Install the [gamescope](https://archlinux.org/packages/?name=gamescope) package:

$ gamescope -f -m 1 -e -- steam -gamepadui

This runs Steam in "big picture" mode (actually Steam Deck mode), in fullscreen, without scaling (i.e. at full resolution). The same settings should also propagate to games run under Steam.

Another option is to configure your compositor to prevent Xwayland from scaling applications entirely. For example, [Hyprland](https://wiki.archlinux.org/title/Hyprland "Hyprland") users can add

xwayland {
  force_zero_scaling = true
}

to the hyprland.conf file to prevent Xwayland from scaling any applications. Note that **all** applications that use Xwayland will stop scaling, and so on HiDPI displays, text and other elements in those applications may become too small to be comfortably viewed.

### Steam flicker/blink with black screen not loading Store/Library or other pages

