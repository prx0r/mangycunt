# Other Repos Roadmap

This file records how the research/reference repos should feed the DAW build.

## Do Not Vendor Everything

Most of these projects are not drop-in dependencies. Treat them as mechanisms to port or interoperate with after the OpenDaw foundation works.

## Priority

### P0: OpenDaw

Purpose: real DAW infrastructure.

Use for:

- Qt6 desktop GUI
- Tracktion Engine/JUCE audio graph
- timeline
- mixer
- piano roll
- VST3 hosting
- project save/load
- audio export

### P1: daw-mcp

Purpose: agent control model.

Use for:

- JSON-RPC style state inspection
- typed DAW tools
- batch changes
- transport/edit/export operations
- music theory helper calls from an agent

The local SoundWorld `POST /commands`, `GET /state`, and `POST /llm` endpoints are the prototype. The OpenDaw version should target project tracks, clips, plugins, and exports.

### P1: Surge XT and Cardinal

Purpose: actual sound design.

Use through OpenDaw plugin hosting rather than embedding their source.

Validation target:

```text
agent creates MIDI clip -> Surge/Cardinal renders sound -> OpenDaw exports WAV
```

### P2: Tonal / Strudel

Purpose: symbolic composition.

Port the useful ideas, not the whole JS stack:

- notes
- intervals
- scales
- chords
- progressions
- voice-leading distance
- Euclidean rhythm
- pattern query by time span

The Rust `soundworld/src/explorer.rs` module already has first-pass harmony/path/pattern ideas.

### P2: dissonant

Purpose: tension scoring.

Use for:

- roughness over spectra
- dissonance-aware chord choice
- tension curves for ambient arrangement

Current SoundWorld roughness is pitch-class approximation only. The better DAW implementation should score the actual synth patch partials where possible.

### P2: projectM

Purpose: visuals.

Use for:

- equation-driven audio-reactive visuals
- state variables driven by harmony, spectral centroid, roughness, density, and motion

Do this after the DAW can already make and export sound.

### P3: flucoma-core

Purpose: audio feature extraction.

Use for:

- corpus analysis
- timbre clustering
- transient/spectral descriptors
- similarity search over user samples

Status: local clone failed previously. Do not build on it until a clean clone exists.

### P3: CLAP tooling

Purpose: future plugin support beyond VST3.

OpenDaw already has VST3 support in scope. CLAP can wait unless VST3 scanning fails for a critical plugin.

## Mathematical Core For AI Composition

The LLM should choose goals and constraints; deterministic code should do the music math.

Suggested primitives:

```text
PitchClassSet
Chord
Scale
Voicing
VoiceLeadingDistance
RoughnessScore
TensionCurve
Pattern<T>
EuclideanRhythm
ArrangementSection
```

Example:

```text
user: start an ambient track, grayscale visuals, no drums
LLM:
  mood = ambient
  density = low
  brightness = low-mid
  harmony = modal
  rhythm = sparse
deterministic planner:
  choose D dorian
  choose chord path by low voice-leading distance
  keep roughness under threshold except transition points
  create MIDI notes
  automate filter cutoff slowly
  render WAV
```

This keeps the system controllable and testable. The LLM does not need to know every MIDI note; it needs a command surface with musical abstractions.
