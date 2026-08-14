use chrono::Local;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Command, Event, EventLog, EventOrigin, InstrumentId, ProjectId, TrackId, Transport};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub id: ProjectId,
    pub name: String,
    pub transport: Transport,
    pub tracks: Vec<Track>,
    pub instruments: InstrumentRegistry,
    pub sounds: SoundLibrary,
    pub harmony: HarmonyState,
    pub world: WorldState,
    pub visual: VisualProject,
    pub automation: AutomationGraph,
    pub history: EventLog,
}

impl Project {
    pub fn new(name: impl Into<String>, sample_rate: u32, bpm: f64) -> Self {
        let mut transport = Transport::default();
        transport.sample_rate = sample_rate;
        transport.bpm = bpm;
        let instrument = InstrumentId::new();
        Self {
            id: ProjectId::new(),
            name: name.into(),
            transport,
            tracks: vec![Track {
                id: TrackId::new(),
                name: "Bass".into(),
                instrument: Some(instrument),
                volume: 0.85,
                pan: 0.0,
                visual_role: VisualRole::Bass,
            }],
            instruments: InstrumentRegistry {
                instruments: vec![InstrumentSlot {
                    id: instrument,
                    name: "MangySynth".into(),
                    kind: InstrumentKind::NativeSynth,
                }],
            },
            sounds: SoundLibrary { count: 1 },
            harmony: HarmonyState::default(),
            world: WorldState::default(),
            visual: VisualProject::default(),
            automation: AutomationGraph::default(),
            history: EventLog::default(),
        }
    }

    pub fn accept_command(&mut self, origin: EventOrigin, command: Command) {
        apply_command(self, &command);
        let event = Event {
            id: Uuid::new_v4(),
            timestamp_samples: self.transport.sample_position,
            musical_time: self.transport.beats(),
            wall_time: Local::now(),
            origin,
            command,
            before_hash: String::new(),
            after_hash: String::new(),
        };
        self.history.push(event);
    }
}

fn apply_command(project: &mut Project, command: &Command) {
    match command {
        Command::Transport(super::TransportCommand::Play) => project.transport.playing = true,
        Command::Transport(super::TransportCommand::Stop) => project.transport.playing = false,
        Command::Transport(super::TransportCommand::SetBpm(bpm)) => project.transport.bpm = *bpm,
        Command::Track(super::TrackCommand::AddTrack { name }) => project.tracks.push(Track {
            id: TrackId::new(),
            name: name.clone(),
            instrument: None,
            volume: 0.85,
            pan: 0.0,
            visual_role: VisualRole::Motif,
        }),
        Command::Music(super::MusicCommand::SetDensity(v)) => project.world.density = *v,
        Command::Music(super::MusicCommand::SetTension(v)) => project.harmony.tension = *v,
        Command::Music(super::MusicCommand::SetMovement(v)) => project.world.movement = *v,
        Command::Music(super::MusicCommand::Nudge { target, delta, .. }) => {
            project.world.last_nudge = Some(format!("{target}:{delta:+.3}"));
        }
        Command::World(super::WorldCommand::Explore { radius, .. }) => {
            project.world.exploration_radius = *radius;
        }
        Command::Visual(super::VisualCommand::SetScene { name }) => {
            project.visual.scene = name.clone();
        }
        _ => {}
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Track {
    pub id: TrackId,
    pub name: String,
    pub instrument: Option<InstrumentId>,
    pub volume: f32,
    pub pan: f32,
    pub visual_role: VisualRole,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VisualRole {
    Bass,
    Pad,
    Motif,
    Percussion,
    Drone,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstrumentRegistry {
    pub instruments: Vec<InstrumentSlot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InstrumentSlot {
    pub id: InstrumentId,
    pub name: String,
    pub kind: InstrumentKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InstrumentKind {
    NativeSynth,
    SampleInstrument,
    ExternalClapInstrument,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundLibrary {
    pub count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HarmonyState {
    pub tonal_center: f32,
    pub tension: f32,
    pub voice_leading_distance: f32,
}

impl Default for HarmonyState {
    fn default() -> Self {
        Self {
            tonal_center: 0.0,
            tension: 0.25,
            voice_leading_distance: 0.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldState {
    pub density: f32,
    pub movement: f32,
    pub novelty: f32,
    pub exploration_radius: f32,
    pub last_nudge: Option<String>,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            density: 0.35,
            movement: 0.35,
            novelty: 0.18,
            exploration_radius: 0.35,
            last_nudge: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisualProject {
    pub scene: String,
}

impl Default for VisualProject {
    fn default() -> Self {
        Self {
            scene: "harmonic_orbits_low".into(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AutomationGraph {
    pub lane_count: usize,
}
