# Topic 5: CS2 Low-Resolution Mode and Minimal Setup

## 1. CS2 at 384x288 Resolution

**Yes, CS2 can be launched at 384x288** using launch parameters. No enforced minimum resolution.

```
-window -w 384 -h 288 -noborder
```

- 4:3 ratio, extremely pixelated but functional
- Launch params override in-game settings minimums
- GPU load dramatically reduced (fewer pixels to render)

## 2. All Relevant Launch Options

```
-window -w 384 -h 288 -noborder -novid -nojoy -nohltv -softparticlesdefaultoff -forcenovsync +fps_max 20 +fps_max_menu 5 +mat_queue_mode 2 +r_dynamic 0 +mat_disable_fancy_blending 1
```

| Option | Status | Effect |
|---|---|---|
| `-window` / `-windowed` / `-sw` | Works | Windowed mode |
| `-w N -h N` | Works | Set resolution |
| `-noborder` | Works | Borderless window |
| `-novid` | Works | Skip intro video |
| `-nojoy` | Works | Disable gamepad support |
| `-nohltv` | Works | Disable GOTV relay |
| `-softparticlesdefaultoff` | Works | Disable depth-feathered particles |
| `-forcenovsync` | Works | Force VSync off |
| `+fps_max N` | Works | Cap FPS (use 15–30 for idling) |
| `+mat_queue_mode 2` | Works | Multicore rendering |
| `+r_dynamic 0` | Works | Disable dynamic lighting |
| `-threads N` | Works but discouraged | Source 2 auto-manages threads |
| `-high` | Works | High process priority |
| `-d3d9ex` | Obsolete | CS2 uses DX11/Vulkan |
| `-noreactlogin` | Dead (Steam flag) | Removed from Steam |
| `-no-browser` | Dead | Removed with Panorama UI |

## 3. CS2 Drop/Farming System (Current as of 2026)

**CS2 uses a Weekly Care Package system** requiring active gameplay:
- Must earn XP via official matchmaking modes
- Each profile level-up = 5,000 XP
- Once per week (resets Wednesday 00:00 UTC), leveling up unlocks a Care Package: choose 2 items from 4 options
- **Prime Status required** for drops (cases/skins)
- AFK/idle players earn minimal/zero XP
- ~30 minutes of legitimate play per week is sufficient for one drop

**Pure idling for drops is NOT viable** in the current system.

## 4. CS2 GPU Requirements

CS2 **requires a real GPU** with Vulkan support (Linux) or DirectX 11 (Windows).
- No software rendering path exists in CS2
- Will refuse to launch without compliant GPU driver
- Minimum: AMD GCN+ or NVIDIA Kepler+ on Linux

On a headless Linux server:
- Xvfb provides virtual display but NOT GPU emulation
- Still need actual Vulkan-capable GPU or Venus virtual GPU (QEMU 9.2+)
- Low-end GPU like GTX 750 Ti (~$20 used) is practical minimum for headless farm

## 5. Window Mode Performance in CS2

| Mode | FPS | Notes |
|---|---|---|
| Exclusive Fullscreen | Baseline | Direct flip, lowest latency |
| Borderless Windowed | **10–30% HIGHER** than exclusive FS | Source 2 engine quirk |
| Small windowed (384x288) | Much higher FPS | Fewer pixels to render |

Borderless windowed is **counterintuitively faster** in CS2 due to Source 2 engine specifics.

## 6. Steam Autostart and Auto-Login

### Linux Autostart

```ini
# ~/.config/autostart/steam.desktop
[Desktop Entry]
Type=Application
Name=Steam
Exec=steam -silent -nofriendsui
X-GNOME-Autostart-enabled=true
```

Or systemd user service:
```ini
[Service]
ExecStart=/bin/bash /home/user/start-steam.sh
Restart=always
```

### Auto-Login Status

**`steam -login username password` is BROKEN** as of mid-2023.

Working approach:
1. Log in once manually with "Remember Me" checked
2. Use saved session cookie for subsequent launches
3. Use `xdotool` or script to re-enter credentials if session expires

### Minimal Steam Flags (still working)

```
steam -silent -nofriendsui -nochatui
```

- `-silent` — start minimized to tray
- `-nofriendsui` — disable friends panel
- `-nochatui` — disable chat tab

## 7. Headless CS2 on Linux (VM Setup)

```bash
# Install virtual display
sudo apt install xvfb

# Start virtual display
Xvfb :99 -screen 0 1024x768x24 &
export DISPLAY=:99

# Start Steam
steam -silent -nofriendsui &
sleep 15

# Launch CS2 at 384x288
steam steam://rungameid/730 -- -window -w 384 -h 288 -noborder -novid +fps_max 20
```

For persistent setup: systemd service wrapping the above.

## Sources
- https://totalcsgo.com/launch-options
- https://tradeit.gg/blog/cs2-drop-pool/
- https://community.skin.club/en/news/cs2-fullscreen-vs-fullscreen-windowed-fps-test
- https://store.steampowered.com/app/730/CounterStrike_2/
- https://gist.github.com/joshuaboniface/50690ad188df15033c5f04b3cac31845
