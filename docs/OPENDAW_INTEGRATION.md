# OpenDaw Integration Plan

Date: 2026-08-15

This repo is pivoting from a standalone SoundWorld prototype toward a proper DAW stack.

## Decision

Use OpenDaw as the DAW foundation, not the Rust SoundWorld app.

```text
OpenDaw
  Qt6 UI
  Tracktion Engine / JUCE audio backend
  VST3 host
  timeline, mixer, piano roll, export
        ^
        |
SoundWorld / Mangy
  AI-native control layer
  harmony/vector exploration
  generated MIDI/automation ideas
  optional visual instruments
```

SoundWorld should become an agent/controller/instrument subsystem that can drive a real DAW. It should not try to recreate timeline editing, VST hosting, audio devices, export, and plugin scanning from scratch.

## Local Build Requirements

For the full captured local machine profile, read `docs/LOCAL_BUILD_ENVIRONMENT.md`.

The Linux host needs:

```bash
sudo apt-get update
sudo apt-get install -y \
  cmake ninja-build git pkg-config \
  qt6-base-dev qt6-base-dev-tools \
  libasound2-dev libjack-jackd2-dev \
  libfreetype-dev libfontconfig1-dev \
  libx11-dev libxext-dev libxinerama-dev libxrandr-dev \
  libxcursor-dev libxcomposite-dev \
  libavformat-dev libavcodec-dev libavutil-dev \
  libswscale-dev libswresample-dev
```

Verified locally on 2026-08-15:

```text
cmake 3.31.6
ninja 1.12.1
Qt6Widgets/Core/Gui 6.8.2
```

## Reproducible Setup

Run:

```bash
cd /tmp/mangycunt-soundworld
./scripts/setup-opendaw-stack.sh
```

Expected result:

```text
external/OpenDaw/
external/OpenDaw/libs/JUCE/
external/OpenDaw/libs/tracktion_engine/
external/OpenDaw/build-linux/
```

The script:

1. Clones OpenDaw into `external/OpenDaw`.
2. Initializes JUCE and Tracktion Engine submodules.
3. Pins JUCE to OpenDaw's documented commit `7c89e11f6b7316c369f3d3f22227c60e816e738b`.
4. Configures a Linux Ninja build against system Qt6.
5. Builds the `OpenDaw` target.

## Current Validation Result

The first local attempt used the ignored reference clone at:

```text
/tmp/mangycunt-soundworld/references/4daw/OpenDaw
```

`git submodule update --init --depth 1` started, but GitHub dropped the large transfer:

```text
error: RPC failed; curl 92 HTTP/2 stream 5 was not closed cleanly: CANCEL
fatal: early EOF
fatal: clone of 'https://github.com/juce-framework/JUCE.git' into submodule path ... failed
```

Tracktion then stalled with no further output and the command was stopped manually. Both submodule directories were empty afterward, so no honest OpenDaw compile was possible yet.

This is a network/bootstrap blocker, not proof that OpenDaw cannot build on this machine.

## Linux Patch Targets To Check

OpenDaw's upstream `CMakeLists.txt` currently contains a Windows Qt default:

```cmake
set(CMAKE_PREFIX_PATH "c:/qt/6.10.2/msvc2022_64" CACHE STRING "Qt prefix path")
```

The setup script passes:

```bash
-DCMAKE_PREFIX_PATH=/usr/lib/x86_64-linux-gnu/cmake/Qt6
```

If CMake still picks the Windows path, patch OpenDaw locally to only set that default under `WIN32`.

Also inspect audio settings code. The README still documents Windows driver names in the UI; Linux should expose ALSA/JACK/PipeWire through JUCE/Tracktion rather than WASAPI/ASIO/DirectSound.

## Plugin Goal: Surge XT and Cardinal

The immediate DAW goal is to load:

```text
Surge XT VST3 or CLAP
Cardinal VST3
```

Known local plugin paths from earlier system inventory:

```text
~/.vst3/Surge XT.vst3
~/.lv2/Surge XT.lv2
~/.clap/Surge XT.clap
~/.vst3/Cardinal.vst3
~/.vst3/CardinalSynth.vst3
~/.vst3/CardinalFX.vst3
~/.lv2/CardinalMini.lv2
```

OpenDaw already has VST3 scanning and instrument/effect selection in its source tree. After OpenDaw builds:

1. Launch OpenDaw.
2. Open plugin scan.
3. Ensure `~/.vst3` is included.
4. Scan.
5. Add a MIDI track.
6. Insert Surge XT.
7. Insert CardinalSynth.
8. Create a MIDI clip and verify sound.
9. Export a short WAV and verify nonzero samples.

## Other Repos: What To Integrate

The other repos are references, not code to vendor blindly.

Use them like this:

```text
daw-mcp
  JSON-RPC/MCP control surface for DAW state, transport, notes, mixer, export

tonal
  music theory API shape for notes, scales, chords, progressions, voice-leading

strudel
  pattern/time model for algorithmic generation

dissonant
  psychoacoustic roughness/dissonance scoring

projectM
  equation-driven visuals tied to audio/harmony state

clap-host
  future CLAP inspection/hosting reference

flucoma-core
  future audio feature extraction; prior clone failed and has not been inspected
```

Concrete integration order:

1. Build OpenDaw on Linux.
2. Verify audio device output and VST3 sound with Surge XT.
3. Add an OpenDaw-local agent API inspired by `daw-mcp`.
4. Port the current SoundWorld typed command API onto OpenDaw project state.
5. Add music-theory helpers from the existing Rust explorer module.
6. Add WAV render validation as a test command.
7. Add visuals only after sound/export/plugin hosting are working.

## Agent API Shape

Expose a localhost-only API first:

```text
GET  /health
GET  /state
POST /commands
POST /llm
POST /render
```

Minimum command set:

```json
[
  {"transport":"play"},
  {"transport":"stop"},
  {"track":{"create":"midi","name":"Surge"}},
  {"plugin":{"scan":true}},
  {"plugin":{"insert":"Surge XT","track":"Surge"}},
  {"clip":{"create_midi":"Surge","bars":8}},
  {"notes":{"track":"Surge","scale":"D dorian","density":0.25}},
  {"export":{"path":"/tmp/mangy-opendaw-test.wav","bars":8}}
]
```

Safety rules:

- localhost only by default
- no shell execution through the API
- typed commands only
- undo boundary for every command batch
- `POST /llm` translates text to typed commands; it does not directly mutate files or call plugins

## Definition Of Done

This integration is not done until all of these pass:

```text
OpenDaw builds on Linux
OpenDaw launches from desktop
audio device opens
Surge XT is scanned
Cardinal is scanned
MIDI clip through Surge makes audible/nonzero output
MIDI clip through Cardinal makes audible/nonzero output
8-bar WAV export contains nonzero finite samples
agent API can create a short ambient project and render it
docs explain how to open and use it
```
