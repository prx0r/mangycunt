use serde::{Deserialize, Serialize};

use super::{InstrumentId, ParamId, PatchId, TrackId};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Transport(TransportCommand),
    Track(TrackCommand),
    Instrument(InstrumentCommand),
    Sound(SoundCommand),
    Music(MusicCommand),
    World(WorldCommand),
    Visual(VisualCommand),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TransportCommand {
    Play,
    Stop,
    SetBpm(f64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TrackCommand {
    AddTrack { name: String },
    SetVolume { track: TrackId, value: f32 },
    SetPan { track: TrackId, value: f32 },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum InstrumentCommand {
    SetParameter {
        instrument: InstrumentId,
        param: ParamId,
        value: f32,
    },
    NoteOn {
        instrument: InstrumentId,
        midi: u8,
        velocity: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SoundCommand {
    SetParam {
        instrument: InstrumentId,
        param: ParamId,
        value: f32,
    },
    RampParam {
        instrument: InstrumentId,
        param: ParamId,
        target: f32,
        beats: f32,
    },
    Mutate {
        patch: PatchId,
        radius: f32,
    },
    Anchor {
        patch: PatchId,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MusicCommand {
    Nudge {
        target: String,
        delta: f32,
        beats: f32,
    },
    SetDensity(f32),
    SetTension(f32),
    SetMovement(f32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum WorldCommand {
    Explore { patch: PatchId, radius: f32 },
    SelectCandidate { patch: PatchId },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VisualCommand {
    SetScene {
        name: String,
    },
    SetBinding {
        source: String,
        target: String,
        amount: f32,
    },
}
