# Creative Corpus And Worlds

`daw3` reframes SoundWorld/Mangy as a growing model of the user's creative library.

The important idea:

```text
patches + samples + motifs + loops + modulations + harmony paths + performances
      -> CreativeObjects
      -> multiple feature views
      -> queryable Worlds
      -> preference history
      -> better exploration
```

This is not one giant latent space. Mangy should support many overlapping worlds.

Examples:

```text
Bass World
Drone World
Metallic World
Track-12 World
Voice Gesture World
```

A sound can belong to several worlds.

## Implemented Now

The first scaffolding lives in:

```text
src/world/mod.rs
```

Types added:

- `CreativeObject`
- `CreativeObjectKind`
- `FeatureView`
- `PatchFeatures`
- `AudioFeatures`
- `RhythmFeatures`
- `MelodyFeatures`
- `HarmonyFeatures`
- `ModulationFeatures`
- `PreferenceFeatures`
- `World`
- `WorldConstraints`
- `Region`
- `PreferenceEvent`
- `CreativeCorpus`

The running app now has:

```rust
corpus: CreativeCorpus
```

Current patches are registered into the corpus.

Anchoring a patch creates a strong preference event.

Saved `project.json` includes:

```json
"creative_corpus": {}
```

## Local Commands Added

The command bar now recognizes:

```text
create bass world
make bass world
show me something
show me something i haven't noticed
```

These are early local placeholders for the future LLM-assisted explorer.

## Future Tool Ideas

Eventually an AI planner should get tools like:

```text
FindSimilar
FindDistant
FindBridges
FindUnderexploredRegion
ExplainCluster
CompareSounds
CreateWorldFromSelection
ExploreBetween
ExploreAround
ExploreAwayFrom
FindUnexpectedRelationship
```

The LLM should not hallucinate poetic descriptions. It should inspect numerical relationships computed by Mangy.

## Next Engineering Step

Replace heuristic map positions with descriptors:

```text
RMS
spectral centroid
spectral spread
rolloff
flatness
roughness
low/mid/high energy
attack
decay
modulation rate
pitch stability
```

Then add:

```rust
trait EmbeddingProvider {
    fn embed(&self, sound: &RenderedSound) -> Vec<f32>;
}
```

Initial providers:

```text
DescriptorEmbedding
PcaEmbedding
```

Later:

```text
CLAP embedding
neural timbre embedding
personal taste embedding
```
