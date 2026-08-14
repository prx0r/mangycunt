use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub timestamp_samples: u64,
    pub musical_time: f64,
    pub wall_time: DateTime<Local>,
    pub origin: EventOrigin,
    pub command: Command,
    pub before_hash: String,
    pub after_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventOrigin {
    HumanUi,
    Midi,
    Voice,
    ProceduralGenerator,
    Ai,
    Automation,
    Replay,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EventLog {
    pub events: Vec<Event>,
}

impl EventLog {
    pub fn push(&mut self, event: Event) {
        self.events.push(event);
    }
}
