# 4daw Reference Pass

Date: 2026-08-15

The file `/home/box/Documents/4daw` was used as the active brief for this pass.

## Local Reference Clones

Shallow clones were placed under:

```text
/tmp/mangycunt-soundworld/references/4daw/
```

These are intentionally ignored by Git via the repository `.gitignore`; do not vendor these third-party repos into Mangy.

Cloned successfully:

- `OpenDaw`: AI/DAW tool architecture, Tracktion-backed DAW checklist, automation and project-state surface.
- `daw-mcp`: MCP/JSON-RPC bridge, DAW state inspection, batch MIDI tools, Tonal.js music analysis, Euclidean rhythm generation.
- `strudel`: reference for time as queryable patterns.
- `tonal`: behavioral reference for music-theory primitives, pitch classes, chords, scales, progressions, voicing, and voice-leading.
- `dissonant`: reference for psychoacoustic roughness/dissonance models over spectra/partials.
- `projectm`: reference for equation-driven music visualization.
- `clap-host`: reference for later CLAP plugin-host inspection.

Attempted but incomplete:

- `flucoma-core`: clone was interrupted by a network disconnect and only a partial `.git` directory was left. Do not treat it as inspected in this pass.

## Mechanisms Extracted

### OpenDaw / daw-mcp

Steal the architecture pattern:

```text
state summary
    +
typed tool surface
    +
batch operations
    +
safety/undo/review boundaries
```

For Mangy this becomes:

```text
GET /state
IntentPlan
Explorer request
CandidateSet
Project/Event commit
```

### Strudel

Steal the time representation:

```text
Pattern<T>
```

A pattern is not just notes. It can hold:

- notes
- patch IDs
- harmony nodes
- visual parameters
- modulation values
- world positions

This pass adds a generic `Pattern<T>` with phase-based querying.

### Tonal

Steal the small pure theory modules and tests:

- pitch class sets
- chord masks
- voice-leading distance
- progressions
- voicing choice by minimal movement

This pass adds a small native Rust `voice_leading_distance` and `HarmonyExplorer` rather than importing JS code.

### Dissonant

Steal the principle:

```text
roughness depends on spectra/partials, not just chord labels
```

This pass still uses a compact pitch-class roughness approximation. Future work should calculate roughness from oscillator partials so the same interval can become more or less tense depending on timbre.

### projectM

Steal the equation-driven visual philosophy:

```text
audio/harmony/timbre state -> evaluated equations -> OpenGL visuals
```

Do not embed projectM directly. Mangy visuals should be driven by:

- harmony roughness
- voice-leading distance
- timbre vector
- motif phase
- world trajectory

## Implemented In This Pass

New source module:

```text
src/explorer.rs
```

Added:

- `CreativeObjectId`
- `CreativeObjectKind`
- `CreativeObjectRecord`
- `FeatureSpace`
- `FeatureVector`
- `WorldSnapshot`
- `Pattern<T>`
- `Motif`
- `HarmonyNode`
- `HarmonyEdge`
- `HarmonyPath`
- `ConstraintSet`
- `FindBridgeRequest`
- `Candidate<T>`
- `CandidateSet<T>`
- `Explorer` trait
- `HarmonyExplorer`
- `voice_leading_distance`
- `chord_roughness`

This is the first concrete version of the architecture from `4daw`:

```text
LLM chooses intent
      ↓
Explorer request
      ↓
Mangy math searches
      ↓
CandidateSet with measured explanations
      ↓
user chooses / preference is recorded
```

## Next Implementation Pass

Build on `src/explorer.rs`:

1. Add `FindBridge` API endpoint.
2. Convert current world candidates into `CandidateSet<Patch>`.
3. Add `Motif` storage to the creative corpus.
4. Add incommensurate `Pattern<T>` periods to ambient generation.
5. Add spectral roughness from oscillator partials.
6. Add visual equations driven by roughness, voice-leading, motif phase, and timbre vector.
