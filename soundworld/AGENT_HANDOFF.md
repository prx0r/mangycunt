# Agent Handoff: SoundWorld / Mangy

Date: 2026-08-15

This file is for an external coding agent continuing the SoundWorld build. It should be read before changing code.

## One-Line Product State

SoundWorld is a lightweight Rust `egui`/`CPAL` standalone audiovisual instrument and AI-control prototype for a low-RAM Linux laptop. It is not a DAW and does not host Surge XT/Cardinal plugins yet.

## Current Local/Repo State

Repository:

```text
https://github.com/prx0r/mangycunt
```

Main project:

```text
soundworld/
```

Known pushed commit after the API/red-team pass:

```text
ada3919
```

Local working clone used during this pass:

```text
/tmp/mangycunt-soundworld
```

User-facing copy:

```text
/home/box/Documents/SoundWorld
```

Installed binary:

```text
/home/box/.local/bin/soundworld
```

Cargo target used because `/tmp` is small:

```text
/root/mangy-cargo-target
```

Build commands:

```bash
cd /tmp/mangycunt-soundworld/soundworld
cargo fmt
CARGO_TARGET_DIR=/root/mangy-cargo-target /root/.cargo/bin/cargo test
CARGO_TARGET_DIR=/root/mangy-cargo-target /root/.cargo/bin/cargo check
CARGO_TARGET_DIR=/root/mangy-cargo-target /root/.cargo/bin/cargo build --release
```

## Implemented Capabilities

### App

- native Rust app using `eframe`/`egui`
- audio output through `cpal`
- bass-oriented internal synth
- patch mutation and 2D candidate map
- ambient note generator
- greyscale procedural visual mode
- project/event graph
- patch/project JSON save paths
- built-in Guide tab
- WAV writer code path

### AI/API Control

The app starts a localhost API server:

```text
127.0.0.1:3769
```

Health:

```bash
curl -s http://127.0.0.1:3769/health
```

Command batch:

```bash
curl -s http://127.0.0.1:3769/commands \
  -H 'Content-Type: application/json' \
  -d '{"origin":"Ai","commands":[{"Transport":"Play"},{"Music":{"SetDensity":0.25}},{"Music":{"SetMovement":0.7}},{"Visual":{"SetScene":{"name":"disabled"}}}]}'
```

State:

```bash
curl -s http://127.0.0.1:3769/state
```

Macro batch:

```bash
curl -s http://127.0.0.1:3769/macro \
  -H 'Content-Type: application/json' \
  -d '{"origin":"Ai","intent":"dark ambient","macros":["ambient","dark","wide","play"]}'
```

Implemented command application covers:

- transport play/stop/BPM
- music density/tension/movement
- semantic nudges
- visual scene enable/disable
- world exploration
- patch mutation
- patch anchoring
- note-on audition
- `GET /state` exposes transport, patch summary, music state, harmony state, visual state, and affect metrics
- `POST /macro` maps simple words into typed commands

Important: `POST /macro` reports rejected unknown macro words. `POST /commands` still accepts valid typed `Command` JSON directly, so it needs a stricter allowlist before exposing it beyond localhost.

## Proof / Validation

See `TESTING_NOTES.md` for exact commands and results.

Validated:

- `cargo fmt`
- 10 unit tests pass
- `cargo check`
- optimized release build
- GUI launch smoke test
- live GUI driven by localhost API using `curl`
- synth DSP produces nonzero finite samples after a MIDI note

Not fully validated:

- physical speaker audibility
- screenshot/pixel validation for visuals
- Ardour/REAPER plugin scan for Surge XT/Cardinal during the latest pass
- real LLM provider calling the API
- strict `POST /commands` allowlist/security hardening

## Architecture Rules

Keep these boundaries:

- realtime audio callback must not call an LLM
- LLM/API must propose typed commands, not shell commands or Rust code
- GUI and API should both feed the same command/event/project layer
- provider calls must be optional
- no API keys in repo
- keep low-RAM laptop constraints in mind
- do not turn this into a full DAW unless explicitly requested

Preferred control architecture:

```text
human text / external agent / LLM provider
        ↓
optional planner
        ↓
validated typed Command values
        ↓
Project.accept_command(...)
        ↓
audio/world/visual/session state
```

## Existing Synth/DAW Context

Surge XT and Cardinal are meant to be used in a real DAW for now.

Known installed/documented plugin paths:

```text
~/.vst3/Surge XT.vst3
~/.lv2/Surge XT.lv2
~/.clap/Surge XT.clap
~/.vst3/Cardinal.vst3
~/.vst3/CardinalSynth.vst3
~/.vst3/CardinalFX.vst3
~/.lv2/CardinalMini.lv2
```

Installed DAW/routing tools documented in `LOCAL_TEST_STATE.md`:

```text
Ardour
PipeWire
pipewire-jack
WirePlumber
qpwgraph
ffmpeg
```

Best next integration is not native plugin hosting. Best next step is SoundWorld as controller:

```text
SoundWorld API / generator
        ↓
MIDI notes + MIDI CC / OSC
        ↓
Ardour or REAPER tracks
        ↓
Surge XT / Cardinal
```

## Extension Backlog

### P0: Harden Existing API

- add explicit allowlist validation before commands are accepted
- reject unknown or unsupported commands with `rejected`
- add tests for invalid JSON, unsupported commands, and command clamping

### P1: Make Agent Control Useful

- add `/audition` endpoint to trigger a note and optionally render a tiny WAV preview
- expand `/macro` with words like `sharp`, `acid`, `clean`, and `dirty`
- add automation commands for filter sweeps and parameter ramps
- add a chat/review panel in the GUI
- add an optional local process provider that can call OpenCode/OpenRouter/OpenAI through external scripts

### P2: DAW/Surge/Cardinal Companion Mode

- add MIDI out with `midir` or ALSA sequencer
- add selectable MIDI output device in GUI
- send notes from ambient generator to external DAW/plugin tracks
- send CC mappings for density, movement, tension, darkness, novelty, energy, and space
- document Ardour and REAPER routing using qpwgraph
- add a minimal project template for Ardour/REAPER if feasible

### P3: Corpus / SoundWorld Expansion

- import WAV renders from Surge/Cardinal
- map WAV files as `CreativeObject`s
- add similarity metadata: RMS, centroid, rough spectral flatness, transient density
- make worlds queryable by tag and feature range
- let API command select/anchor/reuse corpus objects

### P4: Frontier ML / Neural Audio Research Mode

Do not integrate heavy ML into the realtime app first. Use external/offline pipelines and import the results.

Candidate research/tools:

- RAVE: realtime audio variational autoencoder, useful for timbre transformation and neural instrument research. Repo: https://github.com/acids-ircam/RAVE
- Magenta RealTime 2: live AI music model direction; useful reference for text+MIDI/audio interactive control. Site: https://magenta.withgoogle.com/magenta-realtime-2
- nn~ / Max style neural audio plugin patterns: research for DAW-friendly neural inference workflows.
- AudioStellar-style corpus maps: useful conceptual reference for 2D sound corpus exploration.
- pGESAM and latent timbre synthesis repos from earlier spec: treat as research code, not polished plugins.

Integration strategy:

```text
external ML/research process
        ↓
render WAV / features / embeddings
        ↓
SoundWorld corpus import
        ↓
agent explores and controls maps
```

Only attempt live neural inference after the low-spec machine proves it can run the model without breaking audio/GUI responsiveness.

### P5: Native Plugin Hosting

This is high cost. Research only unless explicitly requested.

Likely order:

1. CLAP host prototype
2. LV2 host prototype
3. VST3 only if needed

Risks:

- plugin scanning
- plugin UI embedding
- plugin state save/restore
- crash isolation
- realtime safety
- low-RAM pressure from Cardinal

## Research Pointers

Read:

- `AI_DAW_RESEARCH.md`
- `HARMONY_AGENT_RESEARCH.md`
- `AMBIENT_GEOMETRY_RESEARCH.md`
- `RESEARCH_NOTES.md`
- `API_CONTROL.md`
- `AI_PLANNER.md`
- `CORPUS_WORLDS.md`
- `LOCAL_TEST_STATE.md`
- `TESTING_NOTES.md`

Comparable systems:

- AbleMind: https://ablemind.live/
- Yuma automation in Ableton: https://www.yuma.studio/blog/yuma-writes-automation-in-ableton
- Ableton MCP Extended: https://github.com/uisato/ableton-mcp-extended
- Live MCP: https://live-mcp.mixofreality.studio/
- ChatM4L: https://chatm4l.com/
- nob intelligent synth: https://www.nob.audio/
- Deep Noise DAW plugin guide: https://docs.deepnoise.ai/product-guides/quick-start-tips/working-in-daw
- Rinoa AI DAW plugin beta: https://rinoa.ai/docs/guides/daw

## Good Next PR

Recommended next PR:

```text
Add /state plus API command allowlist
```

Why:

- it makes agent control safer
- it gives an LLM context before acting
- it is small and testable
- it avoids heavy DAW/plugin/ML work too early

Acceptance criteria:

- `GET /state` returns compact JSON for transport, current patch summary, music state, visuals enabled, candidate count, anchors, and event count
- `POST /commands` rejects unsupported commands
- tests prove valid commands still work and invalid commands are rejected
- docs show example agent loop: health -> state -> plan -> commands
