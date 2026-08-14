# Local Test State

Captured: `2026-08-15T00:59:57+07:00`

This file documents the exact local machine and software state used while building and testing SoundWorld/Mangy. External agents should treat this as the target baseline unless a newer file supersedes it.

## Repository State

GitHub repository:

```text
https://github.com/prx0r/mangycunt
```

Local working clone:

```text
/tmp/mangycunt-soundworld
```

Local user-facing source copy:

```text
/home/box/Documents/SoundWorld
```

Current pushed commit at time of capture:

```text
9fb19fc14322eb3f3a4378097045c263bed8d152
```

Working tree at capture:

```text
clean before this document was added
```

SoundWorld project path inside repo:

```text
soundworld/
```

Primary command paths:

```bash
cd /tmp/mangycunt-soundworld/soundworld
/root/.cargo/bin/cargo fmt
/root/.cargo/bin/cargo check
/root/.cargo/bin/cargo build --release
```

Installed runtime binary:

```text
/home/box/.local/bin/soundworld
```

Installed binary size after latest build:

```text
7.5M
```

Desktop launcher:

```text
/home/box/Desktop/SoundWorld.desktop
```

Application menu launcher:

```text
/home/box/.local/share/applications/soundworld.desktop
```

## Hardware

Machine class:

```text
2012-era x86_64 Intel MacBook-style low-RAM laptop
```

Architecture:

```text
x86_64
```

CPU:

```text
Intel(R) Core(TM) i5-3317U CPU @ 1.70GHz
```

CPU topology:

```text
physical cores: 2
threads per core: 2
logical CPUs: 4
max MHz: 2600
min MHz: 800
L3 cache: 3 MiB
```

Important CPU flags:

```text
sse
sse2
ssse3
sse4_1
sse4_2
avx
aes
f16c
rdrand
```

Memory at capture:

```text
Mem total:      3.7 GiB
Mem used:       2.2 GiB
Mem free:       898 MiB
Mem available:  1.5 GiB
Swap total:     3.9 GiB
Swap used:      1.6 GiB
Swap free:      2.3 GiB
```

SoundWorld hardware probe:

```json
{
  "cpu_threads": 4,
  "ram_mb": 3827,
  "audio_device": "default",
  "sample_rate": 48000,
  "recommended_buffer": 512,
  "opengl_version": "queried by egui/glow at runtime",
  "quality_profile": "low"
}
```

Design implication:

```text
Use LOW profile by default.
Prefer 512 sample buffer.
Avoid runtime allocations in audio callbacks.
Avoid heavyweight ML locally.
Keep visuals sharp/basic/greyscale and cheap.
```

## Operating System

Distribution:

```text
Debian GNU/Linux 13 (trixie)
VERSION_ID=13
DEBIAN_VERSION_FULL=13.5
```

Kernel:

```text
Linux debian11 6.12.86+deb13-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.86-1 (2026-05-08) x86_64 GNU/Linux
```

Filesystem/storage:

```text
/      ext4 on /dev/sda2, 160G total, 111G used, 41G available, 74% used
/home  same /dev/sda2 mount
/tmp   tmpfs, 1.9G total, 1.4G used, 495M available, 75% used at capture
swap   /dev/sda3, 3.9G
```

Block layout:

```text
sda      167.7G disk
sda1       976M vfat  /boot/efi
sda2     162.8G ext4  /, /root, /root/.codex
sda3       3.9G swap
```

## Graphics And Audio Hardware

PCI graphics:

```text
Intel Corporation 3rd Gen Core processor Graphics Controller (rev 09)
```

PCI audio:

```text
Intel Corporation 7 Series/C216 Chipset Family High Definition Audio Controller (rev 04)
```

Other relevant PCI devices:

```text
Intel QS77 Express chipset
Broadcom BCM43224 Wi-Fi
Intel DSL3510 Thunderbolt Controller [Cactus Ridge 4C 2012]
```

Graphics target:

```text
eframe/egui with glow/OpenGL backend
No wgpu requirement for the low-spec target
No realtime video encoding
```

## Installed Audio Stack

Debian packages installed:

```text
ardour                1:8.12.0+ds-1
pipewire              1.4.2-1
pipewire-audio        1.4.2-1
pipewire-jack         1.4.2-1
wireplumber           0.5.8-2
qpwgraph              0.8.2-1
ffmpeg                7:7.1.5-0+deb13u1
```

Installed executable paths visible in system `PATH`:

```text
/usr/bin/ardour
/usr/bin/qpwgraph
```

User-local installed executables may not appear in non-login root `PATH`, but exist under:

```text
/home/box/.local/bin/
```

## Installed Build Toolchain

Rust from rustup is primary:

```text
rustc 1.97.1 (8bab26f4f 2026-07-14)
cargo 1.97.1 (c980f4866 2026-06-30)
```

Commands used:

```text
/root/.cargo/bin/rustc --version
/root/.cargo/bin/cargo --version
```

Debian Rust packages are also installed, but rustup currently shadows them:

```text
cargo package: 1.85.0+dfsg3-1
rustc package: 1.85.0+dfsg3-1
```

Native build packages installed:

```text
build-essential
pkg-config
clang                 1:19.0-63
cmake                 3.31.6-2
libasound2-dev        1.2.14-1
libudev-dev           257.13-1~deb13u1
libx11-dev            2:1.8.12-1
libxi-dev             2:1.8.2-1
libgl1-mesa-dev       25.0.7-2+deb13u1
```

## Installed Synths And Plugins

### Surge XT

Version installed:

```text
Surge XT 1.3.4 plugin-only Linux x86_64 archive
```

Downloaded archive:

```text
/tmp/surge-xt-linux-1.3.4-pluginsonly.tar.gz
```

SHA-256:

```text
dd431b75f5fa197c4bffa6ca27ca46970f0a94c834119bb1db7decdeec4c28db
```

Installed plugin paths:

```text
/home/box/.vst3/Surge XT.vst3
/home/box/.vst3/Surge XT Effects.vst3
/home/box/.lv2/Surge XT.lv2
/home/box/.lv2/Surge XT Effects.lv2
/home/box/.clap/Surge XT.clap
/home/box/.clap/Surge XT Effects.clap
```

Installed standalone/CLI paths:

```text
/home/box/.local/bin/Surge XT              30M
/home/box/.local/bin/Surge XT Effects      19M
/home/box/.local/bin/surge-xt-cli          31M
/home/box/.local/bin/surge-xt              symlink to /home/box/.local/bin/Surge XT
```

Desktop launcher:

```text
/home/box/Desktop/Surge XT.desktop
```

### Cardinal

Version installed:

```text
Cardinal 26.02 Linux x86_64 official release
```

Downloaded archive:

```text
/root/downloads/Cardinal-linux-x86_64-26.02.tar.gz
```

SHA-256:

```text
657df0beeec04184de7359cbd3e173a36eeab78077e1e26da81405632f98ec25
```

Installed plugin paths:

```text
/home/box/.vst3/Cardinal.vst3
/home/box/.vst3/CardinalSynth.vst3
/home/box/.vst3/CardinalFX.vst3
/home/box/.lv2/CardinalMini.lv2
```

Installed standalone path:

```text
/home/box/.local/bin/cardinal              101M
```

Desktop launcher:

```text
/home/box/Desktop/Cardinal.desktop
```

### Plugin Directory Sizes

At capture:

```text
/home/box/.vst3   1.1G
/home/box/.lv2    75M
/home/box/.clap   49M
```

Note:

```text
Cardinal full LV2/CLAP bundles are large, so only full VST3 plus CardinalMini LV2 are installed locally.
```

## SoundWorld Runtime State

Installed binary:

```text
/home/box/.local/bin/soundworld            7.5M
```

Source copy for user:

```text
/home/box/Documents/SoundWorld             256K
```

Config path:

```text
/home/box/.config/soundworld/hardware.json
```

Data path:

```text
/home/box/.local/share/soundworld/
```

Default project path:

```text
/home/box/.local/share/soundworld/Default.soundworld/
```

Recordings path:

```text
/home/box/.local/share/soundworld/Default.soundworld/recordings/
```

## Current SoundWorld Features

Implemented:

```text
native Rust egui/CPAL app
hardware probe
low profile assumptions
bass-oriented synth controls
simple CPAL audio output
bounded note queue
background WAV writer
patch JSON saving
2D mutation map
deterministic candidate generation
ambient note generator
typed local command bar
no-visuals command mode
greyscale low-cost visual tab
Guide tab
Project / Command / Event / Transport scaffold
AI planner boundary scaffold
documentation files
```

Important command-bar phrases currently supported:

```text
start ambient
start ambient track
start ambient no visuals
no visuals
visuals
show visuals
enable visuals
disable visuals
darker
brighter
more movement
less movement
more sparse
less dense
more dense
more strange
less strange
more tension
less tension
more space
less space
explore
anchor
anchor this
new bass
```

Not implemented yet:

```text
plugin hosting
MIDI output to DAW
CLAP host
external synth parameter mapping
full audio graph
multi-track timeline
phrase/clip model
event replay
offline video render
live LLM API calls
chat plan review UI
```

## Local Test Commands

Use rustup toolchain explicitly:

```bash
cd /tmp/mangycunt-soundworld/soundworld
/root/.cargo/bin/cargo fmt
/root/.cargo/bin/cargo check
/root/.cargo/bin/cargo build --release
```

Install rebuilt binary:

```bash
cp /tmp/mangycunt-soundworld/soundworld/target/release/soundworld /home/box/.local/bin/soundworld
chown box:box /home/box/.local/bin/soundworld
```

GUI smoke test:

```bash
timeout 8 runuser -u box -- env \
  DISPLAY=:0 \
  XAUTHORITY=/home/box/.Xauthority \
  DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus \
  XDG_RUNTIME_DIR=/run/user/1000 \
  /home/box/.local/bin/soundworld
```

Expected smoke-test result:

```text
Process stays alive until timeout kills it.
Exit code 124 from timeout is expected.
No immediate crash output is expected.
```

Desktop launcher validation:

```bash
desktop-file-validate \
  /home/box/.local/share/applications/soundworld.desktop \
  /home/box/Desktop/SoundWorld.desktop
```

Refresh app menu:

```bash
update-desktop-database /home/box/.local/share/applications
```

Plugin path check:

```bash
find /home/box/.vst3 /home/box/.lv2 /home/box/.clap \
  -maxdepth 2 \
  -iname '*Surge*' -o -iname '*Cardinal*'
```

## Performance Constraints To Preserve

Hard target:

```text
Works acceptably on 4 GB RAM / Intel i5-3317U / integrated Intel graphics.
```

Do:

```text
Keep idle redraw low.
Keep visuals simple and greyscale/sharp by default.
Keep audio callback allocation-free.
Use bounded queues or atomics for audio communication.
Keep project data JSON-first.
Keep LLM/API integrations optional.
Prefer MIDI/CC control of external DAWs before plugin hosting.
```

Do not:

```text
Put LLM calls in realtime audio.
Do plugin scanning in realtime audio.
Require GPU ML.
Require internet to launch.
Require Surge/Cardinal to launch SoundWorld.
Realtime-encode video.
Accidentally turn the first milestone into a full DAW clone.
```

## Recommended Next Local Build Step

The best next feature for this exact machine is:

```text
MIDI output from SoundWorld -> Ardour/REAPER tracks -> Surge XT/Cardinal plugins
```

This gives practical integration with the installed synths without building a plugin host yet.

Implementation target:

```text
Add optional midir dependency.
Add MIDI output device selector.
Send ambient generator notes as MIDI.
Map darkness/movement/density/tension/space to configurable CC messages.
Document qpwgraph/DAW routing.
```

Only after that should CLAP hosting be prototyped.
