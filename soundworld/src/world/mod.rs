use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct CreativeObjectId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct WorldId(pub Uuid);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct EmbeddingSpaceId(pub Uuid);

impl CreativeObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl WorldId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl EmbeddingSpaceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CreativeObjectKind {
    Patch,
    AudioSample,
    Motif,
    Loop,
    ModulationPattern,
    HarmonyPath,
    PerformanceFragment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreativeObject {
    pub id: CreativeObjectId,
    pub kind: CreativeObjectKind,
    pub name: String,
    pub created_at: DateTime<Local>,
    pub tags: Vec<String>,
    pub feature_views: Vec<FeatureView>,
    pub source_ref: String,
}

impl CreativeObject {
    pub fn patch(name: impl Into<String>, patch_id: Uuid, tags: Vec<String>) -> Self {
        Self {
            id: CreativeObjectId::new(),
            kind: CreativeObjectKind::Patch,
            name: name.into(),
            created_at: Local::now(),
            tags,
            feature_views: vec![FeatureView::Patch(PatchFeatures::default())],
            source_ref: format!("patch:{patch_id}"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeatureView {
    Patch(PatchFeatures),
    Audio(AudioFeatures),
    TimbreEmbedding(Vec<f32>),
    Rhythm(RhythmFeatures),
    Melody(MelodyFeatures),
    Harmony(HarmonyFeatures),
    Modulation(ModulationFeatures),
    Preference(PreferenceFeatures),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PatchFeatures {
    pub brightness: f32,
    pub drive: f32,
    pub sub_weight: f32,
    pub movement: f32,
    pub space: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AudioFeatures {
    pub rms: f32,
    pub spectral_centroid: f32,
    pub spectral_spread: f32,
    pub spectral_rolloff: f32,
    pub flatness: f32,
    pub roughness: f32,
    pub low_energy: f32,
    pub mid_energy: f32,
    pub high_energy: f32,
    pub attack: f32,
    pub decay: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RhythmFeatures {
    pub density: f32,
    pub syncopation: f32,
    pub pulse_strength: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MelodyFeatures {
    pub range: f32,
    pub contour: f32,
    pub repetition: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct HarmonyFeatures {
    pub tonal_center: f32,
    pub tension: f32,
    pub voice_leading_distance: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ModulationFeatures {
    pub rate: f32,
    pub depth: f32,
    pub smoothness: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PreferenceFeatures {
    pub audition_count: u32,
    pub selected_count: u32,
    pub anchor_count: u32,
    pub used_in_track_count: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub id: WorldId,
    pub name: String,
    pub members: Vec<CreativeObjectId>,
    pub embedding_spaces: Vec<EmbeddingSpaceId>,
    pub constraints: WorldConstraints,
    pub anchors: Vec<CreativeObjectId>,
    pub discovered_regions: Vec<Region>,
    pub unexplored_regions: Vec<Region>,
}

impl World {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: WorldId::new(),
            name: name.into(),
            members: Vec::new(),
            embedding_spaces: vec![EmbeddingSpaceId::new()],
            constraints: WorldConstraints::default(),
            anchors: Vec::new(),
            discovered_regions: Vec::new(),
            unexplored_regions: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldConstraints {
    pub include_tags: Vec<String>,
    pub exclude_tags: Vec<String>,
    pub preferred_views: Vec<FeatureViewKind>,
    pub novelty: f32,
}

impl Default for WorldConstraints {
    fn default() -> Self {
        Self {
            include_tags: Vec::new(),
            exclude_tags: Vec::new(),
            preferred_views: vec![FeatureViewKind::Patch, FeatureViewKind::Audio],
            novelty: 0.2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FeatureViewKind {
    Patch,
    Audio,
    Timbre,
    Rhythm,
    Melody,
    Harmony,
    Modulation,
    Preference,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Region {
    pub id: Uuid,
    pub label: String,
    pub center: [f32; 2],
    pub radius: f32,
    pub confidence: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PreferenceEvent {
    pub id: Uuid,
    pub created_at: DateTime<Local>,
    pub candidates: Vec<CreativeObjectId>,
    pub selected: Option<CreativeObjectId>,
    pub context: String,
    pub world_position: Option<[f32; 2]>,
    pub weight: PreferenceWeight,
}

impl PreferenceEvent {
    pub fn anchor(selected: CreativeObjectId, context: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Local::now(),
            candidates: vec![selected],
            selected: Some(selected),
            context: context.into(),
            world_position: None,
            weight: PreferenceWeight::Strong,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PreferenceWeight {
    Weak,
    Medium,
    Strong,
    Stronger,
    Strongest,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CreativeCorpus {
    pub objects: Vec<CreativeObject>,
    pub worlds: Vec<World>,
    pub preferences: Vec<PreferenceEvent>,
}

impl CreativeCorpus {
    pub fn ensure_default_world(&mut self) {
        if self.worlds.is_empty() {
            self.worlds.push(World::new("Bass World"));
        }
    }

    pub fn add_object_to_default_world(&mut self, object: CreativeObject) -> CreativeObjectId {
        self.ensure_default_world();
        let id = object.id;
        self.objects.push(object);
        if let Some(world) = self.worlds.first_mut() {
            if !world.members.contains(&id) {
                world.members.push(id);
            }
        }
        id
    }

    pub fn anchor(&mut self, object: CreativeObjectId, context: impl Into<String>) {
        self.ensure_default_world();
        if let Some(world) = self.worlds.first_mut() {
            if !world.anchors.contains(&object) {
                world.anchors.push(object);
            }
        }
        self.preferences
            .push(PreferenceEvent::anchor(object, context));
    }
}
