# Ambient Geometry / Emotion / Visual Research

Date: 2026-08-15

This document answers: what source material should an LLM/agent use to make SoundWorld produce coherent harmony, dissonance curves, ambient structure, and artistic visuals?

Short version: do not build a giant neuroscience simulator. Build a small deterministic music-theory and affect-control layer, then let an LLM choose goals in that layer.

## What The User Seems To Want

Not a full DAW and not a full frontier ML lab.

The real target is closer to:

```text
"Brian Eno-ish generative ambient machine"
        +
"geometry of harmony / dissonance / tension"
        +
"AI agent can ask for mood, harmonic movement, visuals, and sound design"
        +
"outputs editable/recordable audio and visible structure"
```

The agent should be able to say:

```text
make a slow dark ambient piece, low arousal, slightly uneasy, with smooth chord geometry and white-on-black visuals
```

and SoundWorld should translate that into:

- slow BPM
- sparse note density
- minor/modal pitch collection
- low movement
- low brightness
- controlled dissonance
- smooth voice leading
- long envelope/reverb/space
- visual geometry moving slowly with tension and harmonic distance
- WAV recording

## Core Research Sources

### Geometry Of Music

Dmitri Tymoczko's work is the most relevant music-theory spine here.

Useful ideas:

- chords are points in a geometric space
- voice leading is distance between chord points
- smooth progression means short total voice movement
- scales, chords, and voices form nested levels
- macroharmony is the pitch collection active over a longer span

Sources:

- `A Geometry of Music` / Tymoczko overview and reviews: https://mtosmt.org/issues/mto.11.17.3/mto.11.17.3.hook.html
- Tymoczko, `Scale Theory, Serial Theory and Voice Leading`: https://onlinelibrary.wiley.com/doi/abs/10.1111/j.1468-2249.2008.00257.x
- Tymoczko, `Tonality: An Owner's Manual`: https://academic.oup.com/book/57582/chapter-abstract/469157208
- geometry of chord spaces / orbifolds explanation: https://www.math.stonybrook.edu/~tony/whatsnew/oct06/10-2006-media.html
- `Geometry of Music Perception`: https://www.mdpi.com/2227-7390/10/24/4793

SoundWorld use:

```text
chord = point
progression = path
tension = distance from stable region
smoothness = voice-leading distance
visual shape = projection of chord/timbre point
```

### Dissonance / Roughness

For dissonance, use psychoacoustic roughness first, not mystical theory.

Useful models:

- Plomp-Levelt critical bandwidth
- Sethares dissonance curves
- Vassilakis roughness
- Essentia dissonance from spectral peaks

Sources:

- `dissonant` Python package: https://github.com/bzamecnik/dissonant
- Sethares Python gist: https://gist.github.com/endolith/3066664
- Essentia dissonance docs: https://essentia.upf.edu/reference/std_Dissonance.html

SoundWorld use:

```text
dissonance = sum roughness(partial_i, partial_j)
```

Approximate first version:

```text
interval_roughness:
  octave/fifth/fourth = low
  thirds/sixths = medium-low
  seconds/sevenths/tritone = high
```

Then map to visuals:

```text
more dissonance -> more angular deformation, brighter strokes, faster rotation
less dissonance -> smoother circles, dimmer strokes, slower motion
```

### Emotion / Valence-Arousal

Use valence/arousal as control axes, not as claims about true emotion.

Useful datasets:

- DEAM: continuous/static valence and arousal annotations for music
- MusAV: comparative arousal-valence judgments over track previews
- PMEmo and related datasets
- Memo2496
- MERGE / 4Q datasets

Sources:

- DEAM overview: https://www.kaggle.com/datasets/imsparsh/deam-mediaeval-dataset-emotional-analysis-in-music
- DEAM/music2emo loader example: https://huggingface.co/amaai-lab/music2emo/blob/main/dataset_loaders/deam.py
- MusAV: https://mtg.github.io/musav-dataset/
- emotion dataset list: https://github.com/juansgomez87/datasets_emotion
- Memo2496: https://figshare.com/articles/dataset/Memo2496/25827034
- MIR emotion datasets: https://mir.dei.uc.pt/downloads.html

SoundWorld use:

```text
valence  -> consonance, mode, brightness, harmonic stability
arousal  -> tempo, density, attack, movement, visual speed
tension  -> dissonance + harmonic distance + brightness + density
novelty  -> mutation radius + rare intervals + visual asymmetry
```

Do not overclaim neuroscience. Treat it as a usable control surface.

### Predictive Processing / Expectation

The useful musical idea is expectation and surprise:

```text
expected event -> low surprise
unexpected but related event -> interesting novelty
unexpected and unrelated event -> chaos
```

SoundWorld can implement this simply:

```text
surprise = distance(actual_next_chord, predicted_next_chord)
```

Prediction can be deterministic:

- in minor, expect i, iv, v, VI, VII
- after V, expect i or VI
- after long static harmony, allow modal shift
- ambient style prefers slow harmonic rhythm

Use predictive-processing language only as inspiration. The implementation should be cost functions and probability weights.

### Brian Eno / Generative Ambient

The relevant Eno-like mechanism is systems with small rules that produce lots of variation.

Sources:

- Sound Mirrors article on Eno and generative music/visuals: https://repository.falmouth.ac.uk/2727/
- Eno official bio/visual work: https://www.brian-eno.net/about/
- WIRED on `77 Million Paintings`: https://www.wired.com/2007/07/brian-eno-qa-the-infinite-art-of/

SoundWorld use:

```text
few long loops
different cycle lengths
slow random drift
bounded mutation
sparse events
visual layers with independent periods
record the run as the artifact
```

Do not imitate Eno literally. Borrow the system idea:

```text
artist designs rules -> machine unfolds variation
```

## Code Libraries Worth Studying

For music theory:

- Tonal.js: TypeScript music theory library with notes, intervals, chords, scales, keys, Roman numerals, progression, voicing, and voice-leading modules. https://github.com/tonaljs/tonal
- Teoria.js: older lightweight JS music theory library. https://github.com/saebekassebil/teoria
- music21: Python computational musicology toolkit with harmony/chord symbol/Roman numeral analysis. https://music21.org/music21docs/moduleReference/moduleHarmony.html
- music21 source for harmony classes. https://music21.org/music21docs/_modules/music21/harmony.html
- THIRI MCP: deterministic music-theory MCP/API for AI agents. https://github.com/BluesPrince/thiri-mcp
- musicaiz: Python symbolic music generation/analysis with harmony modules. https://github.com/carlosholivan/musicaiz
- midi-mcp: MCP server for AI agents creating/manipulating MIDI. https://github.com/cfogelklou/midi-mcp

For agent/DAW structure:

- Ableton MCP Extended: https://github.com/uisato/ableton-mcp-extended
- Live MCP: https://live-mcp.mixofreality.studio/
- ChatM4L: https://chatm4l.com/
- fl-studio-web-daw MCP architecture: https://github.com/manueltarouca/fl-studio-web-daw

For harmony generation:

- cadenza: deterministic harmonically coherent electronic MIDI stems. https://github.com/Andrea-Cavallo/cadenza
- MIDI-LLM: text-to-MIDI LLM research. https://github.com/slSeanWU/MIDI-LLM

For neural/audio research:

- DDSP: differentiable DSP for interpretable learned synth/audio parameters. https://github.com/magenta/ddsp
- RAVE: realtime neural audio autoencoder. https://github.com/acids-ircam/RAVE
- Magenta RealTime 2: https://magenta.withgoogle.com/magenta-realtime-2

## Minimal Math Engine For SoundWorld

### Pitch Classes

```text
pc = midi % 12
```

### Key Mask

```text
C minor = {0, 2, 3, 5, 7, 8, 10}
```

### Chord

```text
Cm7 = {0, 3, 7, 10}
```

### Harmonic Fit

```text
fit(note, chord, key):
  +1.0 if pc(note) in chord
  +0.5 if pc(note) in key
  -1.0 otherwise
```

### Voice-Leading Distance

For two voicings:

```text
distance(A, B) = min_permutation sum(abs(A_i - B_perm_i))
```

Small distance means smooth movement.

### Harmonic Tension

```text
tension =
  0.35 * dissonance
  + 0.25 * harmonic_distance_from_tonic
  + 0.20 * non_chord_tone_ratio
  + 0.10 * brightness
  + 0.10 * rhythmic_density
```

### Emotion Mapping

```text
arousal =
  0.35 * tempo_norm
  + 0.25 * density
  + 0.20 * attack_sharpness
  + 0.20 * visual_speed

valence =
  0.35 * consonance
  + 0.20 * major_or_bright_mode
  + 0.20 * warmth
  + 0.15 * stability
  - 0.10 * harshness
```

This is not a scientific truth claim. It is a controllable creative mapping.

### Timbre Vector

```text
timbre = [
  sub,
  noise,
  cutoff,
  resonance,
  drive,
  attack,
  release,
  space,
  width
]
```

### Visual Vector

```text
visual = [
  geometry_scale,
  deformation,
  brightness,
  particle_density,
  rotation_speed,
  symmetry,
  depth
]
```

Map:

```text
dissonance -> deformation
tension -> brightness + angularity
arousal -> speed + density
valence -> smoothness + warmth/brightness
voice_leading_distance -> visual jump distance
macroharmony_size -> number of rings/nodes
```

## Agent Prompt Pattern

The LLM should not directly generate notes forever. It should create a plan:

```json
{
  "intent": "dark ambient, low arousal, slightly uneasy",
  "key": "C minor",
  "tempo": 68,
  "chord_grid": ["Cm9", "Abmaj7", "Fm9", "Gsus4"],
  "tension_curve": [0.2, 0.25, 0.45, 0.3],
  "density": 0.22,
  "timbre_macros": ["dark", "subby", "wide"],
  "visual_macros": ["black_white", "slow_orbits", "dissonance_deforms"],
  "record_seconds": 45
}
```

Then deterministic SoundWorld code turns the plan into commands and audio.

## Build Recommendation

Do this next:

1. Add `HarmonyState` to SoundWorld.
2. Add `/state`.
3. Add `/plan` or `/macro` accepting a small JSON plan like above.
4. Add deterministic chord-grid generation in Rust.
5. Add visual mapping from `tension`, `dissonance`, `voice_leading_distance`, and `arousal`.
6. Keep RAVE/DDSP/ML outside the realtime path for now.

This gives the agent real music theory source code to use without overengineering the whole project.
