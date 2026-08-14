# Mangy / SoundWorld Architecture

`daw2` changes the direction from a single-file prototype into a small runtime built around one canonical project graph.

The first implemented step is the `src/core/` layer:

- `ids.rs`: stable typed IDs for projects, tracks, instruments, patches, and parameters.
- `transport.rs`: one canonical clock with sample and beat conversions.
- `command.rs`: typed commands for transport, tracks, instruments, sound, music, world, and visuals.
- `event.rs`: immutable events with origin, musical time, sample time, and command payload.
- `project.rs`: the first canonical `Project` graph: transport, tracks, instruments, sound library, harmony, world, visuals, automation, and event history.
- `src/ai/`: provider-neutral AI planner boundary. LLMs may propose validated commands, but they do not touch realtime audio.

The running egui prototype still owns the current UI/audio state, but key UI actions now also flow into:

```text
Command -> Event -> Project.history
```

This is deliberate. It lets the app stay runnable while the architecture is extracted behind it.

## Current Boundary

Implemented now:

- canonical project model
- universal command enums
- event log and event origins
- transport clock scaffold
- typed track/instrument/project IDs
- UI actions recorded into project history
- project save includes the new graph state

Not implemented yet:

- separate Cargo workspace crates
- audio graph trait and node scheduler
- instrument trait
- CLAP hosting
- phrase/clip model
- visual scene graph
- signal bus
- event replay
- provider adapters for opencode/OpenRouter/OpenAI

## Next Migration

Do not add large features directly to `main.rs`.

Next safe step:

1. Move app/UI code into `src/app.rs`.
2. Move CPAL engine into `src/audio/engine.rs`.
3. Move synth DSP into `src/synth/`.
4. Keep `main.rs` as a small launcher.
5. Add `Instrument` and `AudioNode` traits once code is split.

Only after that should CLAP hosting be prototyped.
