# DAW Validation Notes

Date: 2026-08-15

## What Was Validated

Local Linux build prerequisites:

```text
ninja --version -> 1.12.1
pkg-config --modversion Qt6Widgets Qt6Core Qt6Gui -> 6.8.2
cmake --version -> 3.31.6
```

OpenDaw source reference:

```text
/tmp/mangycunt-soundworld/references/4daw/OpenDaw
commit 3cbc660
```

OpenDaw uses:

```text
Qt6 UI
JUCE
Tracktion Engine
FFmpeg via pkg-config on Linux/macOS
VST3 plugin hosting flags
```

## What Failed

OpenDaw submodule initialization failed due GitHub/network transfer instability.

Command:

```bash
git submodule update --init --depth 1
```

Observed failure:

```text
error: RPC failed; curl 92 HTTP/2 stream 5 was not closed cleanly: CANCEL (err 8)
fatal: early EOF
fatal: clone of 'https://github.com/juce-framework/JUCE.git' into submodule path ... failed
```

After retry scheduling, the Tracktion clone produced no output for several minutes and was stopped manually. `git submodule status` still showed both submodules uninitialized:

```text
-7c89e11f6b7316c369f3d3f22227c60e816e738b libs/JUCE
-2877b621f2fbee564d0696a616b86bf8ba8c8ab0 libs/tracktion_engine
```

Both `libs/JUCE` and `libs/tracktion_engine` were empty, so CMake configure/build was not attempted.

A second retry through the committed setup path also stalled while cloning JUCE:

```bash
cd /tmp/mangycunt-soundworld
./scripts/setup-opendaw-stack.sh
```

Observed output:

```text
Cloning into '/tmp/mangycunt-soundworld/external/OpenDaw'...
Submodule 'libs/JUCE' ... registered for path 'libs/JUCE'
Submodule 'libs/tracktion_engine' ... registered for path 'libs/tracktion_engine'
Cloning into '/tmp/mangycunt-soundworld/external/OpenDaw/libs/JUCE'...
```

The process then produced no progress for multiple 30 second intervals and was stopped manually. The partial `external/` directory is intentionally ignored by Git.

## Why This Matters

The current repository now has a reproducible script and docs for OpenDaw integration, but a full local DAW build is still blocked until the submodules download cleanly.

## Next Validation Command

Run:

```bash
cd /tmp/mangycunt-soundworld
./scripts/setup-opendaw-stack.sh
```

Then validate:

```bash
external/OpenDaw/build-linux/OpenDaw_artefacts/Release/OpenDaw
```

or locate the executable with:

```bash
find external/OpenDaw/build-linux -type f -perm -111 -name 'OpenDaw*' -print
```

## Final Audio Proof Required

The DAW integration is only proven when this exists:

```text
/tmp/mangy-opendaw-test.wav
```

and a WAV inspection confirms:

```text
duration > 0
sample rate is valid
samples are finite
peak amplitude > 0.001
RMS amplitude > 0.0001
```

Target manual flow:

1. Launch OpenDaw.
2. Scan VST3 plugins.
3. Add MIDI track.
4. Insert Surge XT.
5. Draw or agent-generate an 8-bar MIDI clip.
6. Press play and confirm meters move.
7. Export WAV.
8. Repeat with Cardinal.
