use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Transport {
    pub sample_position: u64,
    pub sample_rate: u32,
    pub bpm: f64,
    pub beats_per_bar: u8,
    pub beat_unit: u8,
    pub playing: bool,
}

impl Default for Transport {
    fn default() -> Self {
        Self {
            sample_position: 0,
            sample_rate: 48_000,
            bpm: 82.0,
            beats_per_bar: 4,
            beat_unit: 4,
            playing: false,
        }
    }
}

impl Transport {
    pub fn seconds(&self) -> f64 {
        self.sample_position as f64 / self.sample_rate as f64
    }

    pub fn beats(&self) -> f64 {
        self.seconds() * self.bpm / 60.0
    }
}
