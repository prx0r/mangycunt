# Local Build Environment

Date captured: 2026-08-15

This file describes the exact local machine state used for the Mangy/SoundWorld/OpenDaw work so another machine or external agent can build and test against the same target.

## Machine Target

```text
Hostname/kernel label: debian11
OS: Debian GNU/Linux 13 trixie
Debian full version: 13.5
Kernel: Linux 6.12.86+deb13-amd64
Architecture: x86_64
CPU: Intel Core i5-3317U @ 1.70GHz
CPU topology: 2 cores / 4 threads
CPU max frequency: 2.6 GHz
RAM: 3.7 GiB total
Swap: 3.9 GiB total
Root disk: 160G total, 39G available at capture time
/tmp: tmpfs, 1.9G total, 1.5G available at capture time
```

Important constraint: this is a low-RAM, older laptop-class CPU. Prefer incremental builds, low parallelism if memory pressure appears, and avoid heavy ML training locally.

Recommended build target directory for Rust:

```text
/root/mangy-cargo-target
```

Reason: `/tmp` is small and can fill quickly.

## Repository State

```text
Repository: https://github.com/prx0r/mangycunt
Local working clone: /tmp/mangycunt-soundworld
Current pushed commit at capture: 358c381
Branch: main
```

Ignored local third-party/reference trees:

```text
/tmp/mangycunt-soundworld/references/
/tmp/mangycunt-soundworld/external/
```

These are intentionally ignored so broken or partial third-party clones are not committed.

## System Toolchain

```text
git: 2.47.3
cmake: 3.31.6
ninja: 1.12.1
gcc: 14.2.0
g++: 14.2.0
rustc: 1.97.1
cargo: 1.97.1
```

Qt and FFmpeg development versions reported through `pkg-config`:

```text
Qt6Widgets: 6.8.2
Qt6Core: 6.8.2
Qt6Gui: 6.8.2
libavformat: 61.7.103
libavcodec: 61.19.101
libavutil: 59.39.100
libswscale: 8.3.100
libswresample: 5.3.100
```

## Installed DAW/Audio Packages

Confirmed installed Debian packages include:

```text
ardour 1:8.12.0+ds-1
ardour-data 1:8.12.0+ds-1
ardour-lv2-plugins 1:8.12.0+ds-1
ardour-video-timeline 1:8.12.0+ds-1
ffmpeg 7:7.1.5-0+deb13u1
pipewire 1.4.2-1
pipewire-alsa 1.4.2-1
pipewire-audio 1.4.2-1
pipewire-bin 1.4.2-1
pipewire-jack 1.4.2-1
pipewire-pulse 1.4.2-1
wireplumber 0.5.8-2
qpwgraph 0.8.2-1
libjack-jackd2-0 1.9.22~dfsg-4
libjack-jackd2-dev 1.9.22~dfsg-4
qt6-base-dev 6.8.2+dfsg-9+deb13u2
qt6-base-dev-tools 6.8.2+dfsg-9+deb13u2
```

Runtime version commands:

```text
pipewire --version -> libpipewire 1.4.2
wireplumber --version -> libwireplumber 0.5.8
jackd --version -> command not found
```

Interpretation: PipeWire and PipeWire JACK compatibility libraries are installed, but the `jackd` daemon binary is not installed. The DAW should be tested through JUCE/Tracktion audio device selection against ALSA/PipeWire/PipeWire-JACK as available, not by assuming `jackd` is present.

## OpenDaw Build Dependencies

Dependencies installed for the OpenDaw Linux build attempt:

```bash
sudo apt-get install -y \
  ninja-build \
  qt6-base-dev \
  qt6-base-dev-tools \
  libasound2-dev \
  libjack-jackd2-dev \
  libfreetype-dev \
  libfontconfig1-dev \
  libx11-dev \
  libxext-dev \
  libxinerama-dev \
  libxrandr-dev \
  libxcursor-dev \
  libxcomposite-dev \
  libavformat-dev \
  libavcodec-dev \
  libavutil-dev \
  libswscale-dev \
  libswresample-dev
```

Rebuild command:

```bash
cd /tmp/mangycunt-soundworld
./scripts/setup-opendaw-stack.sh
```

Known blocker at capture: OpenDaw itself cloned, but JUCE/Tracktion submodule download stalled or failed due network transfer instability. See `docs/DAW_VALIDATION.md`.

## Plugin State

Desired synth plugins for validation:

```text
Surge XT
Cardinal
```

Expected plugin locations once installed:

```text
/home/box/.vst3/Surge XT.vst3
/home/box/.lv2/Surge XT.lv2
/home/box/.clap/Surge XT.clap
/home/box/.vst3/Cardinal.vst3
/home/box/.vst3/CardinalSynth.vst3
/home/box/.vst3/CardinalFX.vst3
/home/box/.lv2/CardinalMini.lv2
```

Current verification on 2026-08-15:

```text
find /home/box/.vst3 /home/box/.lv2 /home/box/.clap ... -> no files found
find /home/box -maxdepth 5 -iname '*Surge*' / '*Cardinal*' -> no files found
find /usr/lib /usr/local/lib /usr/share -maxdepth 5 -iname '*Surge*' / '*Cardinal*' -> no files found
dpkg -l | rg -i 'surge|cardinal' -> no packages found
```

So the current machine should be treated as having the DAW/audio stack installed, but Surge XT and Cardinal are not verified on disk at this capture point. Before final DAW validation, install or restore those plugins and then update this file with exact paths.

## Existing SoundWorld Prototype

Known install target from earlier work:

```text
/home/box/.local/bin/soundworld
```

Desktop launchers from earlier work:

```text
/home/box/Desktop/SoundWorld.desktop
/home/box/.local/share/applications/soundworld.desktop
```

The standalone prototype is not the final DAW. Use it as:

```text
AI/control prototype
harmony/vector exploration reference
future OpenDaw agent subsystem source material
```

## Low-RAM Build Guidance

Recommended settings for this machine:

```bash
export CARGO_TARGET_DIR=/root/mangy-cargo-target
cmake --build external/OpenDaw/build-linux --target OpenDaw --parallel 2
```

If the machine starts swapping heavily, reduce to:

```bash
cmake --build external/OpenDaw/build-linux --target OpenDaw --parallel 1
```

Do not run local RAVE/ML model training on this laptop. Use pretrained inference only, or offload training to a GPU machine.

## Reproduction Checklist For Another Machine

1. Use Debian 13 x86_64 or a close Linux distro.
2. Install the OpenDaw dependency package list above.
3. Ensure Qt6, FFmpeg dev libraries, Ninja, CMake, GCC/G++ are available.
4. Clone `https://github.com/prx0r/mangycunt`.
5. Run `./scripts/setup-opendaw-stack.sh`.
6. Install Surge XT and Cardinal VST3 plugins.
7. Launch OpenDaw and scan `~/.vst3`.
8. Create MIDI tracks with Surge XT and Cardinal.
9. Export `/tmp/mangy-opendaw-test.wav`.
10. Validate nonzero audio using `ffprobe` and peak/RMS inspection.

## Final Target Test

The target machine is considered compatible when an agent or developer can run:

```text
Create 8 bars of dark ambient MIDI, insert Surge XT, play it through OpenDaw, export WAV.
```

and produce:

```text
/tmp/mangy-opendaw-test.wav
```

with:

```text
duration > 0
peak amplitude > 0.001
RMS amplitude > 0.0001
```
