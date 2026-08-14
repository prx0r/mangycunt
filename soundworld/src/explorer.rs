#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CreativeObjectId(pub Uuid);

impl CreativeObjectId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum CreativeObjectKind {
    Patch,
    AudioSample,
    Motif,
    Loop,
    HarmonyPath,
    ModulationPattern,
    VisualScene,
    World,
    PerformanceFragment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FeatureVector {
    pub space: FeatureSpace,
    pub values: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum FeatureSpace {
    Timbre,
    Rhythm,
    Harmony,
    Movement,
    Preference,
    Visual,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreativeObjectRecord {
    pub id: CreativeObjectId,
    pub kind: CreativeObjectKind,
    pub name: String,
    pub tags: Vec<String>,
    pub features: Vec<FeatureVector>,
    pub parents: Vec<CreativeObjectId>,
    pub children: Vec<CreativeObjectId>,
    pub anchored: bool,
    pub liked: bool,
    pub rejected: bool,
    pub used: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldSnapshot {
    pub objects: Vec<CreativeObjectRecord>,
    pub anchors: Vec<CreativeObjectId>,
    pub locked_pitch_classes: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate<T> {
    pub item: T,
    pub score: f32,
    pub explanation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CandidateSet<T> {
    pub candidates: Vec<Candidate<T>>,
}

impl<T> CandidateSet<T> {
    pub fn best(&self) -> Option<&Candidate<T>> {
        self.candidates
            .iter()
            .min_by(|a, b| a.score.total_cmp(&b.score))
    }
}

pub trait Explorer {
    type Request;
    type Candidate;

    fn search(
        &self,
        world: &WorldSnapshot,
        request: &Self::Request,
    ) -> CandidateSet<Self::Candidate>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PatternEvent<T> {
    pub start_beats: f32,
    pub duration_beats: f32,
    pub value: T,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pattern<T> {
    pub period_beats: f32,
    pub events: Vec<PatternEvent<T>>,
}

impl<T: Clone> Pattern<T> {
    pub fn values_at(&self, beat: f32) -> Vec<T> {
        if self.period_beats <= 0.0 {
            return Vec::new();
        }
        let phase = beat.rem_euclid(self.period_beats);
        self.events
            .iter()
            .filter(|event| {
                event_contains_phase(
                    phase,
                    event.start_beats,
                    event.duration_beats,
                    self.period_beats,
                )
            })
            .map(|event| event.value.clone())
            .collect()
    }
}

fn event_contains_phase(phase: f32, start: f32, duration: f32, period: f32) -> bool {
    if duration <= 0.0 || period <= 0.0 {
        return false;
    }
    if duration >= period {
        return true;
    }
    let start = start.rem_euclid(period);
    let end = (start + duration).rem_euclid(period);
    if start < end {
        phase >= start && phase < end
    } else {
        phase >= start || phase < end
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NoteEvent {
    pub midi: u8,
    pub velocity: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Motif {
    pub id: CreativeObjectId,
    pub name: String,
    pub notes: Pattern<NoteEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonyNode {
    pub name: String,
    pub pitch_classes: Vec<u8>,
    pub voicing: Vec<i16>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonyEdge {
    pub from: HarmonyNode,
    pub to: HarmonyNode,
    pub voice_leading: f32,
    pub roughness: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonyPath {
    pub nodes: Vec<HarmonyNode>,
    pub edges: Vec<HarmonyEdge>,
    pub total_cost: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConstraintSet {
    pub max_voice_leading: f32,
    pub roughness_ceiling: f32,
    pub preserve_pitch_classes: Vec<u8>,
}

impl Default for ConstraintSet {
    fn default() -> Self {
        Self {
            max_voice_leading: 12.0,
            roughness_ceiling: 1.0,
            preserve_pitch_classes: vec![],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FindBridgeRequest {
    pub from: HarmonyNode,
    pub to: HarmonyNode,
    pub steps: usize,
    pub constraints: ConstraintSet,
}

pub struct HarmonyExplorer;

impl Explorer for HarmonyExplorer {
    type Request = FindBridgeRequest;
    type Candidate = HarmonyPath;

    fn search(
        &self,
        _world: &WorldSnapshot,
        request: &Self::Request,
    ) -> CandidateSet<Self::Candidate> {
        let edge = HarmonyEdge {
            from: request.from.clone(),
            to: request.to.clone(),
            voice_leading: voice_leading_distance(&request.from.voicing, &request.to.voicing),
            roughness: chord_roughness(&request.to.pitch_classes),
        };
        let mut rejected = false;
        if edge.voice_leading > request.constraints.max_voice_leading {
            rejected = true;
        }
        if edge.roughness > request.constraints.roughness_ceiling {
            rejected = true;
        }
        for pc in &request.constraints.preserve_pitch_classes {
            if !request.from.pitch_classes.contains(pc) || !request.to.pitch_classes.contains(pc) {
                rejected = true;
            }
        }
        let cost = edge.voice_leading + edge.roughness * 8.0 + if rejected { 1000.0 } else { 0.0 };
        CandidateSet {
            candidates: vec![Candidate {
                item: HarmonyPath {
                    nodes: vec![request.from.clone(), request.to.clone()],
                    edges: vec![edge.clone()],
                    total_cost: cost,
                },
                score: cost,
                explanation: format!(
                    "voice_leading={:.2}, roughness={:.2}, rejected={}",
                    edge.voice_leading, edge.roughness, rejected
                ),
            }],
        }
    }
}

pub fn voice_leading_distance(a: &[i16], b: &[i16]) -> f32 {
    let mut a = a.to_vec();
    let mut b = b.to_vec();
    a.sort_unstable();
    b.sort_unstable();
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (*x as f32 - *y as f32).abs())
        .sum()
}

pub fn chord_roughness(pitch_classes: &[u8]) -> f32 {
    if pitch_classes.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    let mut pairs = 0.0_f32;
    for i in 0..pitch_classes.len() {
        for j in (i + 1)..pitch_classes.len() {
            let interval = ((pitch_classes[i] as i16 - pitch_classes[j] as i16).abs() % 12) as u8;
            let simple = interval.min(12 - interval);
            total += match simple {
                0 => 0.0,
                5 | 7 => 0.08,
                3 | 4 | 8 | 9 => 0.22,
                2 | 10 => 0.55,
                1 | 6 | 11 => 0.85,
                _ => 0.35,
            };
            pairs += 1.0;
        }
    }
    (total / pairs).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pattern_queries_values_by_phase() {
        let pattern = Pattern {
            period_beats: 5.0,
            events: vec![
                PatternEvent {
                    start_beats: 0.0,
                    duration_beats: 1.0,
                    value: "a",
                },
                PatternEvent {
                    start_beats: 3.0,
                    duration_beats: 1.0,
                    value: "b",
                },
            ],
        };

        assert_eq!(pattern.values_at(0.2), vec!["a"]);
        assert_eq!(pattern.values_at(3.5), vec!["b"]);
        assert_eq!(pattern.values_at(5.2), vec!["a"]);
        assert!(pattern.values_at(2.0).is_empty());
    }

    #[test]
    fn pattern_handles_events_wrapping_period_boundary() {
        let pattern = Pattern {
            period_beats: 5.0,
            events: vec![PatternEvent {
                start_beats: 4.5,
                duration_beats: 1.0,
                value: "wrap",
            }],
        };

        assert_eq!(pattern.values_at(4.75), vec!["wrap"]);
        assert_eq!(pattern.values_at(5.25), vec!["wrap"]);
        assert!(pattern.values_at(3.0).is_empty());
    }

    #[test]
    fn voice_leading_distance_prefers_small_motion() {
        let cmaj = vec![60, 64, 67];
        let amin = vec![60, 64, 69];
        let far = vec![72, 76, 79];

        assert!(voice_leading_distance(&cmaj, &amin) < voice_leading_distance(&cmaj, &far));
    }

    #[test]
    fn harmony_explorer_scores_bridge_with_constraints() {
        let explorer = HarmonyExplorer;
        let request = FindBridgeRequest {
            from: HarmonyNode {
                name: "Cm9".into(),
                pitch_classes: vec![0, 2, 3, 7, 10],
                voicing: vec![48, 55, 62, 63],
            },
            to: HarmonyNode {
                name: "Abmaj7".into(),
                pitch_classes: vec![8, 0, 3, 7],
                voicing: vec![48, 56, 63, 67],
            },
            steps: 2,
            constraints: ConstraintSet {
                max_voice_leading: 16.0,
                roughness_ceiling: 0.7,
                preserve_pitch_classes: vec![0],
            },
        };
        let world = WorldSnapshot {
            objects: vec![],
            anchors: vec![],
            locked_pitch_classes: vec![0],
        };

        let results = explorer.search(&world, &request);
        let best = results.best().unwrap();

        assert!(best.score < 1000.0);
        assert_eq!(best.item.nodes.len(), 2);
        assert!(best.explanation.contains("voice_leading"));
    }
}
