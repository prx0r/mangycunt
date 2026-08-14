#![allow(dead_code)]

use serde::{Deserialize, Serialize};

use crate::core::Command;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiRequest {
    pub provider: AiProvider,
    pub user_text: String,
    pub project_summary: ProjectSummary,
    pub allowed_tools: Vec<AiTool>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AiProvider {
    Disabled,
    OpenCode,
    OpenRouter,
    OpenAi,
    LocalProcess { command: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub bpm: f64,
    pub playing: bool,
    pub tracks: usize,
    pub instruments: usize,
    pub recent_events: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AiTool {
    InspectProject,
    SetParameter,
    RampParameter,
    ExploreSound,
    AnchorSound,
    SetDensity,
    SetTension,
    SetMovement,
    ChangeVisualScene,
    ScheduleEvent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AiPlan {
    pub summary: String,
    pub commands: Vec<Command>,
    pub confidence: f32,
}

pub trait AiPlanner {
    fn available(&self) -> bool;
    fn propose(&self, request: AiRequest) -> anyhow::Result<AiPlan>;
}

pub struct DisabledAiPlanner;

impl AiPlanner for DisabledAiPlanner {
    fn available(&self) -> bool {
        false
    }

    fn propose(&self, _request: AiRequest) -> anyhow::Result<AiPlan> {
        anyhow::bail!("AI planner is disabled")
    }
}
