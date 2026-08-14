# SoundWorld User Guide

SoundWorld is a standalone experimental instrument. It is meant for:

```text
make sound -> explore variations -> start ambient generation -> nudge -> record
```

It is not a finished DAW. Surge XT and Cardinal are separate synths you can use in Ardour/REAPER right now.

## Launch

From the desktop:

- Open `SoundWorld` from the application menu, or
- double-click the `SoundWorld` desktop shortcut.

From terminal:

```bash
soundworld
```

From source:

```bash
cd soundworld
scripts/run.sh
```

## First 5 Minutes

1. Open the `SYNTH` tab.
2. Press `Audition C2`.
3. Move `cutoff`, `drive`, `sub`, `Oscillator A level`, and `Oscillator B level`.
4. Press `Save patch` when the sound is useful.
5. Open `WORLD`.
6. Click a node to audition a variation.
7. Press `Anchor` if you like it.
8. Press `Generate Ambient`.
9. Type `darker`, `more movement`, or `less dense` into the bottom command bar.
10. Press `Record WAV` to capture audio.

## Tabs

### SYNTH

Manual bass sound design.

Controls:

- Oscillator A/B wave
- Oscillator levels
- fine tuning
- pulse width
- sub level
- noise
- filter cutoff
- resonance
- drive
- envelope
- space

Useful workflow:

```text
Audition C2
  -> shape oscillator/body/filter
  -> Save patch
  -> Explore
```

### WORLD

2D mutation map.

What it does now:

- Generates 16 deterministic variations around the current patch.
- Shows them as nodes.
- Clicking a node auditions that patch.
- `Anchor` keeps the current patch.

Controls:

- `radius`: how far variations move from the current sound.
- `novelty`: how strange/random the mutation can become.

### TRACK

Simple ambient generator controls.

Controls:

- `bpm`
- `density`
- `tension`
- `movement`
- `novelty`
- `energy`

This is not a timeline yet. It is a continuous seeded generator using the current patch.

### VISUAL

Sharp low-cost greyscale visuals.

The current visual is a simple harmonic-orbit renderer driven by:

- energy
- movement
- tension
- novelty
- bass/sub weight
- filter brightness

Use `Disable visuals` if you only want sound.

### SESSION

Project state and recording info.

Shows:

- patch count
- anchor count
- event count
- transport state
- visual state
- data path

Use `Save project` to write JSON state.

### GUIDE

Built-in quick reference.

This tab is the in-app reminder for how to play the instrument and what the command bar supports.

## Command Bar

The bottom text field is the local command bar. It is intentionally shaped like a future chatbar.

Works now:

```text
start ambient
start ambient track
start ambient no visuals
no visuals
visuals
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
create bass world
show me something
show me something i haven't noticed
```

Example:

```text
start ambient no visuals
```

This starts the ambient generator, switches away from the visual tab, and disables visual drawing.

## Recording

Press:

```text
Record WAV
```

Press it again to stop.

Recordings are written to:

```text
~/.local/share/soundworld/Default.soundworld/recordings/
```

Project JSON is written under:

```text
~/.local/share/soundworld/Default.soundworld/
```

Hardware probe:

```text
~/.config/soundworld/hardware.json
```

## Surge XT And Cardinal

These are installed for DAW use.

Surge XT:

```text
~/.vst3/Surge XT.vst3
~/.lv2/Surge XT.lv2
~/.clap/Surge XT.clap
```

Cardinal:

```text
~/.vst3/Cardinal.vst3
~/.vst3/CardinalSynth.vst3
~/.vst3/CardinalFX.vst3
~/.lv2/CardinalMini.lv2
```

Use them in Ardour/REAPER by rescanning plugins.

SoundWorld does not host these plugins yet. The likely next step is MIDI/CC control:

```text
SoundWorld command/world generator
      -> MIDI notes / CC automation
      -> Ardour/REAPER tracks
      -> Surge XT / Cardinal
```

That is much easier and lighter than immediately building a plugin host.

## AI-Native Plan

Yes, SoundWorld can become AI-native.

The important rule:

```text
LLM proposes commands.
SoundWorld validates and schedules them.
Audio thread stays deterministic and safe.
```

Good future prompts:

```text
start ambient track with these anchored sounds, no visuals
make it darker over 16 bars
use the heavy bass as the center and make sparse pulses around it
generate four variations but keep the sub weight
```

These should become typed internal commands:

```text
Command -> Event -> Project
```

The current command bar already points in that direction. Today it has a local parser. Later it can call an opencode/OpenRouter/OpenAI/local provider adapter.

## What Is Built Now

Built:

- standalone Rust app
- CPAL audio output
- simple bass synth
- low-cost greyscale visuals
- deterministic mutation candidates
- command bar
- WAV recorder
- JSON project save
- canonical `Project`, `Command`, `Event`, `Transport` scaffolding
- AI planner boundary docs/code
- creative corpus/world/preference scaffolding

Not built yet:

- plugin hosting
- full DAW timeline
- CLAP hosting
- true multi-track audio graph
- MIDI output to DAW
- LLM provider calls
- offline video renderer
- event replay

## Recommended Next Build Steps

1. Add MIDI output so SoundWorld can drive Surge XT/Cardinal in Ardour.
2. Add a visible AI/chat panel that shows proposed commands before applying them.
3. Split `main.rs` into app/audio/synth/visual modules.
4. Add phrase/clip generator state.
5. Add a real signal bus for visuals.
6. Prototype CLAP hosting only after the app architecture is stable.
