# SoundWorld Research Notes

This file records the working questions behind SoundWorld and where to research next.

## Current User Questions

- What is SoundWorld?
- Can SoundWorld use Surge XT and Cardinal?
- Do we need to build a DAW?
- Can Surge XT and Cardinal be used in a DAW right now?
- How do we keep the project fast on a low-RAM 2012-era Linux laptop?
- How do we make the visuals sharp, basic, mostly greyscale, and not fuzzy?
- How do we make a built-in user guide?

## Short Answers

SoundWorld is currently a standalone Rust audiovisual instrument prototype. It is not a DAW and does not host plugins yet.

Surge XT and Cardinal can already be used in normal Linux DAWs such as Ardour or REAPER. They are installed as plugins under the user plugin folders:

- VST3: `~/.vst3`
- LV2: `~/.lv2`
- CLAP: `~/.clap`

Using Surge XT and Cardinal inside SoundWorld is possible, but it is not the easiest first step. SoundWorld would need one of these integration strategies:

1. External routing with PipeWire/JACK.
2. MIDI/audio companion mode.
3. Native plugin hosting.
4. Rebuilding selected ideas from Surge/Cardinal inside SoundWorld.

The most pragmatic path is external routing first, then plugin hosting later if the standalone instrument proves worth expanding.

## Integration Options

### Option A: External Routing

Run SoundWorld, Surge XT standalone, Cardinal standalone, and Ardour/REAPER side by side.

Use PipeWire/JACK routing through `qpwgraph`.

Pros:

- Fastest to make usable.
- No plugin host code inside SoundWorld.
- Keeps SoundWorld lightweight.
- Works with existing Surge/Cardinal apps.

Cons:

- More manual routing.
- Session recall is weaker.
- SoundWorld cannot directly save the internal Surge/Cardinal patch state.

Research:

- PipeWire JACK compatibility.
- `qpwgraph` routing.
- MIDI output from Rust via `midir`.
- JACK audio/MIDI APIs.

### Option B: SoundWorld Sends MIDI To DAW/Plugins

SoundWorld remains the world/nudge/generator controller.

Ardour/REAPER hosts Surge XT and Cardinal.

SoundWorld sends MIDI notes/CC automation to the DAW or plugin tracks.

Pros:

- Much easier than hosting plugins.
- Keeps the DAW responsible for plugin scanning and audio stability.
- SoundWorld can become the performance brain.

Cons:

- Requires MIDI routing setup.
- Patch save/recall remains split between SoundWorld and the DAW.

Research:

- Rust `midir`.
- ALSA sequencer MIDI.
- PipeWire MIDI routing.
- Ardour MIDI learn and automation.
- REAPER ReaControlMIDI / MIDI CC routing.

### Option C: Native Plugin Hosting

SoundWorld loads VST3/LV2/CLAP plugins directly.

Pros:

- Best integrated user experience.
- SoundWorld could host Surge/Cardinal in its own graph.
- Better long-term session recall if plugin state is saved correctly.

Cons:

- This is effectively beginning to build a small DAW/plugin host.
- Plugin scanning, UI embedding, state save/restore, realtime safety, and crash isolation are non-trivial.
- Cardinal is heavy; hosting it inside a low-RAM app may conflict with the original hardware target.

Research:

- CLAP host API.
- LV2 host API and Lilv.
- VST3 SDK licensing and hosting examples.
- `nih-plug` ecosystem.
- `baseview` / plugin editor embedding.
- Plugin sandboxing and crash recovery.
- Ardour, Carla, and Zrythm host architectures.

### Option D: Rebuild A Small Native Modular/Synth Layer

Instead of hosting Cardinal, SoundWorld implements a small native modular patch system and selected bass-focused DSP modules.

Pros:

- Most aligned with the lightweight spec.
- Best performance control.
- Full deterministic project save.
- No plugin-host complexity.

Cons:

- More DSP work over time.
- Does not instantly reuse existing Surge/Cardinal patches.

Research:

- Surge XT DSP architecture and license.
- Cardinal/VCV module concepts.
- `fundsp`, `dasp`, and Rust audio DSP crates.
- State-variable filters, wavetable oscillators, modulation matrices.
- Patch graph scheduling.

## Recommended Roadmap

### M0: Use Existing DAWs Now

Use Ardour or REAPER to make sounds with Surge XT and Cardinal immediately.

Goal:

- Make basses in Surge XT.
- Make modular patches in Cardinal.
- Record/render good sounds into a folder.
- Feed the best ideas into SoundWorld later.

### M1: SoundWorld MIDI Out

Add MIDI output from SoundWorld so it can drive Surge/Cardinal tracks in Ardour/REAPER.

This gives the feeling of SoundWorld controlling external synths without hosting them.

Tasks:

- Add `midir`.
- Select MIDI output device.
- Send note on/off from the ambient generator.
- Send CC messages for darkness, movement, density, tension, novelty, energy, and space.
- Add a built-in routing guide.

### M2: PipeWire/JACK Companion Mode

Add explicit setup docs and possibly helper scripts for:

- launching SoundWorld,
- launching Ardour/REAPER,
- opening `qpwgraph`,
- routing MIDI/audio.

### M3: Internal Sample/Render Layer

Let SoundWorld import WAV renders from Surge/Cardinal and place them on the SoundWorld map.

This keeps the custom SoundWorld interaction while using external synths for sound creation.

### M4: Plugin Hosting Research Prototype

Only after the instrument interaction is clearly useful, prototype one plugin format:

1. CLAP first if feasible.
2. LV2 second for Linux-native integration.
3. VST3 only if there is a clear need.

Do not make the main app depend on plugin hosting until the prototype is reliable.

## Installed Synths And DAW Use

Surge XT:

- Standalone app: `Surge XT`
- VST3: `~/.vst3/Surge XT.vst3`
- LV2: `~/.lv2/Surge XT.lv2`
- CLAP: `~/.clap/Surge XT.clap`

Cardinal:

- Standalone app: `Cardinal`
- VST3: `~/.vst3/Cardinal.vst3`
- VST3 synth: `~/.vst3/CardinalSynth.vst3`
- VST3 FX: `~/.vst3/CardinalFX.vst3`
- Small LV2: `~/.lv2/CardinalMini.lv2`

Ardour:

- Installed from Debian.
- Use plugin scan/rescan to find Surge XT and Cardinal.

PipeWire / qpwgraph:

- Installed for audio routing.
- Use `qpwgraph` to connect standalone apps and DAW tracks.

## Research Links

- Surge XT releases: https://github.com/surge-synthesizer/releases-xt/releases
- Surge XT source: https://github.com/surge-synthesizer/surge
- Cardinal: https://github.com/DISTRHO/Cardinal
- Ardour: https://github.com/Ardour/ardour
- Carla plugin host: https://github.com/falkTX/Carla
- CLAP: https://github.com/free-audio/clap
- LV2: https://lv2plug.in/
- Lilv LV2 host library: https://drobilla.net/software/lilv.html
- VST3 SDK: https://github.com/steinbergmedia/vst3sdk
- nih-plug: https://github.com/robbert-vdh/nih-plug
- CPAL: https://github.com/RustAudio/cpal
- midir: https://github.com/Boddlnagg/midir
- PipeWire: https://pipewire.org/
- qpwgraph: https://gitlab.freedesktop.org/rncbc/qpwgraph

## Design Constraint Reminder

Do not accidentally build a full DAW first.

The near-term useful version is:

SoundWorld as controller/instrument + DAW hosting Surge/Cardinal + PipeWire/MIDI routing.

Full plugin hosting can come later after the interaction proves itself.
