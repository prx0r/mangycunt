# Harmony-Aware Agent Research

Date: 2026-08-15

This note is for building SoundWorld into something an AI agent can control intelligently, not just randomly nudge.

## Useful GitHub / Research Directions

- musicaiz: symbolic music generation, evaluation, and analysis with harmony modules for chords, intervals, and keys. Source: https://github.com/carlosholivan/musicaiz
- MIDI-LLM: text-to-MIDI generation by adapting LLMs to MIDI token vocabularies. Useful as a reference, but likely too heavy for the target laptop. Source: https://github.com/slSeanWU/MIDI-LLM
- DDSP: differentiable digital signal processing. Useful for learning interpretable synth/audio parameters, not for direct low-spec realtime integration yet. Source: https://github.com/magenta/ddsp
- matchmaker: realtime music alignment with audio/MIDI features such as chroma and pitch/chord streams. Useful for following or analyzing performed material. Source: https://github.com/pymatchmaker/matchmaker
- weavemuse: agentic music AI system combining harmony analysis, symbolic generation, audio generation, and format conversion. Useful as a broad architecture reference. Source: https://github.com/manoskary/weavemuse
- midi-mcp: MCP server that gives agents MIDI composition/manipulation tools, including chord progressions and voice leading. Source: https://github.com/cfogelklou/midi-mcp
- cadenza: harmonically coherent MIDI generation for electronic music stems. Useful reference for deterministic key/BPM/chord/stem generation. Source: https://github.com/Andrea-Cavallo/cadenza
- fl-studio-web-daw: web DAW with MCP tools for state, sequencer, piano roll, sound design, mixer, automation, transport, persistence, and WAV export. Source: https://github.com/manueltarouca/fl-studio-web-daw
- When-in-Rome: functional harmonic analysis corpus/tools. Useful if SoundWorld later needs Roman numeral / functional harmony reasoning. Source: https://github.com/MarkGotham/When-in-Rome

## Mathematical Model To Build

The agent needs a compact musical state representation. Do not ask an LLM to infer all music theory from prose every frame.

Represent a session as:

```text
S = {
  key,
  mode,
  tempo,
  meter,
  chord_grid,
  scale_degrees,
  pitch_class_histogram,
  active_notes,
  bass_register,
  tension,
  density,
  movement,
  timbre_vector,
  energy,
  section
}
```

### Pitch And Harmony

Use pitch classes mod 12:

```text
pc = midi_note % 12
```

Represent a chord as a pitch-class set:

```text
Cmaj7 = {0, 4, 7, 11}
Cm7   = {0, 3, 7, 10}
```

Represent a key as a tonic plus scale mask:

```text
C minor = tonic 0, mask {0, 2, 3, 5, 7, 8, 10}
```

Consonance / fit can start simple:

```text
fit(note, chord, key) =
  +1.0 if note pitch class is in chord
  +0.5 if note pitch class is in key
  -1.0 otherwise
```

Then improve it with interval weights:

```text
unison/octave: high stability
perfect fifth: stable
third/sixth: consonant color
second/seventh/tritone: tension
```

### Voice Leading

To move from chord A to chord B, minimize total pitch movement:

```text
cost(A -> B) = sum_i abs(note_i_next - note_i_current)
```

Add penalties for:

- notes outside instrument range
- too much parallel motion if the style wants smoother harmony
- bass jumping too far unless a section change asks for it
- melody crossing below bass

For electronic music, this does not need classical strictness. The practical goal is coherence:

```text
same key + shared chord grid + small voice-leading cost + stable bass roots
```

### Tension Curve

Make tension explicit instead of vague.

Inputs:

- non-chord tones
- dissonant intervals
- filter cutoff / brightness
- rhythmic density
- syncopation
- harmonic distance from tonic
- section position

Example:

```text
tension =
  0.30 * harmonic_distance
  + 0.25 * dissonance
  + 0.20 * rhythmic_density
  + 0.15 * brightness
  + 0.10 * novelty
```

Then the agent can say:

```text
increase tension over 8 bars
```

and SoundWorld maps that to:

- more non-chord passing tones
- brighter filter
- higher movement
- denser notes
- more visual deformation

### Timbre Vector

Represent synth sound as a normalized vector:

```text
timbre = [
  sub_level,
  osc_b_level,
  noise_level,
  cutoff,
  resonance,
  drive,
  attack,
  release,
  space,
  width
]
```

Prompt macros become vector moves:

```text
dark    -> cutoff down, sub up, brightness down
heavy   -> sub up, drive up, attack down
sharp   -> attack down, cutoff up, resonance up
wide    -> width up, space up
acid    -> resonance up, cutoff movement up, drive up
ambient -> attack up, release up, space up, density down
```

This is close to how prompt-to-patch synth products work: the AI does not need to invent DSP; it selects parameter moves in a constrained space.

## Agent Tool Surface

Next agent-facing tools should be:

```text
get_state
set_transport
set_harmony
set_chord_grid
add_notes
set_macro
add_automation
audition
record_wav
save_project
```

For SoundWorld's current API, this means adding:

- `GET /state`
- `POST /macro`
- `POST /recording/start`
- `POST /recording/stop`
- `POST /audition`
- `POST /automation`

## Practical Build Plan

1. Add `/state` so an agent can inspect key, BPM, density, movement, current patch, candidates, and event count.
2. Add a `HarmonyState` struct with tonic, mode, current chord, and chord grid.
3. Add deterministic chord progression generation for minor electronic music.
4. Add macro-to-timbre mapping.
5. Add MIDI-out companion mode so SoundWorld can drive Surge/Cardinal in Ardour.
6. Add optional Python/offline analysis adapters later for musicaiz or similar libraries.

Keep the first version deterministic and inspectable. Use LLMs to choose goals and plans; use math/code to enforce musical validity.
