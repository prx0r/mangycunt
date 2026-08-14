use std::{
    f32::consts::TAU,
    fs::{self, File},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use chrono::Local;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam_channel::{bounded, Receiver, Sender};
use eframe::egui;
use hound::{SampleFormat, WavSpec, WavWriter};
use nalgebra::Vector2;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod ai;
mod core;
mod world;
use crate::core::{
    Command, EventOrigin, InstrumentCommand, MusicCommand, ParamId, Project, SoundCommand,
    TransportCommand, VisualCommand, WorldCommand,
};
use crate::world::{CreativeCorpus, CreativeObject};

const API_BIND: &str = "127.0.0.1:3769";

fn main() -> eframe::Result<()> {
    let hardware = HardwareProbe::probe();
    let _ = hardware.write();
    let audio = AudioEngine::start().ok();
    let api_state = Arc::new(Mutex::new(ApiStateSnapshot::default()));
    let api_rx = start_api_server(API_BIND, api_state.clone()).ok();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1180.0, 820.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SoundWorld",
        options,
        Box::new(move |_cc| Box::new(SoundWorldApp::new(hardware, audio, api_rx, api_state))),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ApiCommandRequest {
    #[serde(default = "default_api_origin")]
    origin: EventOrigin,
    commands: Vec<Command>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ApiCommandResponse {
    accepted: usize,
    rejected: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ApiMacroRequest {
    #[serde(default = "default_api_origin")]
    origin: EventOrigin,
    #[serde(default)]
    intent: String,
    macros: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ApiMacroResponse {
    accepted: usize,
    macros: Vec<String>,
    commands: Vec<Command>,
    rejected: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ApiStateSnapshot {
    service: String,
    playing: bool,
    recording: bool,
    visuals_enabled: bool,
    mode: String,
    event_count: usize,
    candidate_count: usize,
    anchor_count: usize,
    patch: PatchSummary,
    music: MusicState,
    harmony: HarmonyState,
    visual: VisualState,
    affect: AffectState,
}

impl Default for ApiStateSnapshot {
    fn default() -> Self {
        Self {
            service: "soundworld".into(),
            playing: false,
            recording: false,
            visuals_enabled: true,
            mode: "World".into(),
            event_count: 0,
            candidate_count: 0,
            anchor_count: 0,
            patch: PatchSummary::default(),
            music: MusicState::default(),
            harmony: HarmonyState::default(),
            visual: VisualState::default(),
            affect: AffectState::default(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct PatchSummary {
    id: Uuid,
    name: String,
    sub_level: f32,
    cutoff: f32,
    resonance: f32,
    drive: f32,
    attack: f32,
    release: f32,
    space: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AffectState {
    valence: f32,
    arousal: f32,
    tension: f32,
    dissonance: f32,
    voice_leading_distance: f32,
}

impl Default for AffectState {
    fn default() -> Self {
        Self {
            valence: 0.5,
            arousal: 0.35,
            tension: 0.25,
            dissonance: 0.2,
            voice_leading_distance: 0.0,
        }
    }
}

fn default_api_origin() -> EventOrigin {
    EventOrigin::Ai
}

fn start_api_server(
    bind: &str,
    state: Arc<Mutex<ApiStateSnapshot>>,
) -> Result<Receiver<ApiCommandRequest>> {
    let listener = TcpListener::bind(bind).with_context(|| format!("bind API server at {bind}"))?;
    let (tx, rx) = bounded::<ApiCommandRequest>(64);
    let bind_label = bind.to_string();
    thread::Builder::new()
        .name("soundworld-api".into())
        .spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => handle_api_stream(stream, &tx, &state),
                    Err(e) => eprintln!("SoundWorld API accept error on {bind_label}: {e}"),
                }
            }
        })
        .context("spawn API server thread")?;
    Ok(rx)
}

fn handle_api_stream(
    mut stream: TcpStream,
    tx: &Sender<ApiCommandRequest>,
    state: &Arc<Mutex<ApiStateSnapshot>>,
) {
    let result = read_api_request(&mut stream).and_then(|(method, path, body)| {
        if method == "GET" && path == "/health" {
            return Ok((
                200,
                serde_json::json!({ "ok": true, "service": "soundworld" }),
            ));
        }
        if method == "GET" && path == "/state" {
            let snapshot = state.lock().map(|state| state.clone()).unwrap_or_default();
            return Ok((200, serde_json::to_value(snapshot)?));
        }
        if method == "POST" && path == "/commands" {
            let request: ApiCommandRequest =
                serde_json::from_slice(&body).context("invalid command JSON")?;
            let accepted = request.commands.len();
            tx.send(request).context("send commands to app")?;
            let response = ApiCommandResponse {
                accepted,
                rejected: vec![],
            };
            return Ok((200, serde_json::to_value(response)?));
        }
        if method == "POST" && path == "/macro" {
            let request: ApiMacroRequest =
                serde_json::from_slice(&body).context("invalid macro JSON")?;
            let (commands, rejected) = macros_to_commands(&request.macros);
            let accepted = commands.len();
            if accepted > 0 {
                tx.send(ApiCommandRequest {
                    origin: request.origin,
                    commands: commands.clone(),
                })
                .context("send macro commands to app")?;
            }
            let response = ApiMacroResponse {
                accepted,
                macros: request.macros,
                commands,
                rejected,
            };
            return Ok((200, serde_json::to_value(response)?));
        }
        Ok((404, serde_json::json!({ "error": "not found" })))
    });

    let (status, payload) = match result {
        Ok(value) => value,
        Err(e) => (400, serde_json::json!({ "error": e.to_string() })),
    };
    let body = serde_json::to_vec(&payload).unwrap_or_else(|_| b"{\"error\":\"encode\"}".to_vec());
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Bad Request",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(&body);
}

fn read_api_request(stream: &mut TcpStream) -> Result<(String, String, Vec<u8>)> {
    stream.set_read_timeout(Some(Duration::from_secs(2))).ok();
    let mut reader = BufReader::new(stream);
    let mut first = String::new();
    reader.read_line(&mut first)?;
    let mut parts = first.split_whitespace();
    let method = parts.next().context("missing HTTP method")?.to_string();
    let path = parts.next().context("missing HTTP path")?.to_string();
    let mut content_length = 0_usize;
    loop {
        let mut line = String::new();
        reader.read_line(&mut line)?;
        if line == "\r\n" || line.is_empty() {
            break;
        }
        if let Some(value) = line
            .strip_prefix("Content-Length:")
            .or_else(|| line.strip_prefix("content-length:"))
        {
            content_length = value.trim().parse().context("bad Content-Length")?;
        }
    }
    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader.read_exact(&mut body)?;
    }
    Ok((method, path, body))
}

fn macros_to_commands(macros: &[String]) -> (Vec<Command>, Vec<String>) {
    let mut commands = Vec::new();
    let mut rejected = Vec::new();
    for name in macros {
        match name.trim().to_lowercase().as_str() {
            "ambient" | "eno" | "slow_ambient" => {
                commands.push(Command::Transport(TransportCommand::SetBpm(68.0)));
                commands.push(Command::Music(MusicCommand::SetDensity(0.22)));
                commands.push(Command::Music(MusicCommand::SetMovement(0.22)));
                commands.push(Command::Music(MusicCommand::SetTension(0.25)));
                commands.push(Command::Music(MusicCommand::Nudge {
                    target: "space".into(),
                    delta: 0.35,
                    beats: 16.0,
                }));
            }
            "dark" | "darker" => commands.push(Command::Music(MusicCommand::Nudge {
                target: "darkness".into(),
                delta: 0.2,
                beats: 8.0,
            })),
            "bright" | "brighter" => commands.push(Command::Music(MusicCommand::Nudge {
                target: "darkness".into(),
                delta: -0.2,
                beats: 8.0,
            })),
            "subby" | "heavy" => {
                commands.push(Command::Music(MusicCommand::Nudge {
                    target: "darkness".into(),
                    delta: 0.1,
                    beats: 8.0,
                }));
                commands.push(Command::Music(MusicCommand::SetDensity(0.3)));
            }
            "wide" | "space" => commands.push(Command::Music(MusicCommand::Nudge {
                target: "space".into(),
                delta: 0.25,
                beats: 8.0,
            })),
            "tense" | "uneasy" => {
                commands.push(Command::Music(MusicCommand::SetTension(0.65)));
                commands.push(Command::Music(MusicCommand::SetMovement(0.55)));
            }
            "calm" | "low_arousal" => {
                commands.push(Command::Transport(TransportCommand::SetBpm(62.0)));
                commands.push(Command::Music(MusicCommand::SetDensity(0.16)));
                commands.push(Command::Music(MusicCommand::SetMovement(0.14)));
                commands.push(Command::Music(MusicCommand::SetTension(0.18)));
            }
            "no_visuals" => commands.push(Command::Visual(VisualCommand::SetScene {
                name: "disabled".into(),
            })),
            "visuals" | "black_white" | "slow_orbits" => {
                commands.push(Command::Visual(VisualCommand::SetScene {
                    name: "harmonic_orbits_low".into(),
                }));
            }
            "play" | "start" => commands.push(Command::Transport(TransportCommand::Play)),
            "stop" => commands.push(Command::Transport(TransportCommand::Stop)),
            _ => rejected.push(name.clone()),
        }
    }
    (commands, rejected)
}

#[derive(Clone, Serialize, Deserialize)]
struct HardwareProbe {
    cpu_threads: usize,
    ram_mb: u64,
    audio_device: String,
    sample_rate: u32,
    recommended_buffer: u32,
    opengl_version: String,
    quality_profile: String,
}

impl HardwareProbe {
    fn probe() -> Self {
        let host = cpal::default_host();
        let audio_device = host
            .default_output_device()
            .and_then(|d| d.name().ok())
            .unwrap_or_else(|| "unknown".into());
        Self {
            cpu_threads: std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1),
            ram_mb: read_mem_mb().unwrap_or(4096),
            audio_device,
            sample_rate: 48_000,
            recommended_buffer: 512,
            opengl_version: "queried by egui/glow at runtime".into(),
            quality_profile: "low".into(),
        }
    }

    fn write(&self) -> Result<()> {
        let path = config_dir().join("hardware.json");
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }
}

fn read_mem_mb() -> Option<u64> {
    let meminfo = fs::read_to_string("/proc/meminfo").ok()?;
    let kb = meminfo
        .lines()
        .find_map(|line| line.strip_prefix("MemTotal:"))?
        .split_whitespace()
        .next()?
        .parse::<u64>()
        .ok()?;
    Some(kb / 1024)
}

fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".config"))
        .join("soundworld")
}

fn data_dir() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home_dir().join(".local/share"))
        .join("soundworld")
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Synth,
    World,
    Track,
    Visual,
    Session,
    Guide,
}

#[derive(Clone, Serialize, Deserialize)]
struct Patch {
    id: Uuid,
    name: String,
    created_at: chrono::DateTime<chrono::Local>,
    osc_a: OscillatorState,
    osc_b: OscillatorState,
    sub_level: f32,
    noise_level: f32,
    filter: FilterState,
    amp: AmpState,
    effects: EffectsState,
    semantic: SemanticState,
    parent: Option<Uuid>,
    generation: u32,
    seed: u64,
    tags: Vec<String>,
}

impl Patch {
    fn init_bass() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "bass-001".into(),
            created_at: Local::now(),
            osc_a: OscillatorState {
                wave: Wave::Saw,
                pitch_octave: -2,
                fine: 0.0,
                level: 0.78,
                pulse_width: 0.5,
                wavetable_pos: 0.2,
            },
            osc_b: OscillatorState {
                wave: Wave::Square,
                pitch_octave: -2,
                fine: -0.04,
                level: 0.28,
                pulse_width: 0.42,
                wavetable_pos: 0.0,
            },
            sub_level: 0.55,
            noise_level: 0.015,
            filter: FilterState {
                mode: FilterMode::LowPass,
                cutoff: 0.33,
                resonance: 0.18,
                drive: 0.24,
                key_tracking: 0.2,
            },
            amp: AmpState {
                attack: 0.006,
                decay: 0.12,
                sustain: 0.68,
                release: 0.18,
            },
            effects: EffectsState {
                space: 0.18,
                width: 0.22,
            },
            semantic: SemanticState {
                darkness: 0.45,
                movement: 0.32,
                density: 0.35,
                tension: 0.25,
                novelty: 0.15,
                energy: 0.55,
                space: 0.18,
            },
            parent: None,
            generation: 0,
            seed: 91832,
            tags: vec!["bass".into()],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct OscillatorState {
    wave: Wave,
    pitch_octave: i32,
    fine: f32,
    level: f32,
    pulse_width: f32,
    wavetable_pos: f32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
enum Wave {
    Sine,
    Triangle,
    Saw,
    Square,
    Wavetable,
}

#[derive(Clone, Serialize, Deserialize)]
struct FilterState {
    mode: FilterMode,
    cutoff: f32,
    resonance: f32,
    drive: f32,
    key_tracking: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
enum FilterMode {
    LowPass,
    BandPass,
    HighPass,
    Notch,
}

#[derive(Clone, Serialize, Deserialize)]
struct AmpState {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

#[derive(Clone, Serialize, Deserialize)]
struct EffectsState {
    space: f32,
    width: f32,
}

#[derive(Clone, Serialize, Deserialize)]
struct SemanticState {
    darkness: f32,
    movement: f32,
    density: f32,
    tension: f32,
    novelty: f32,
    energy: f32,
    space: f32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Candidate {
    patch: Patch,
    pos: Vector2<f32>,
    rms: f32,
    centroid: f32,
}

#[derive(Clone, Serialize, Deserialize)]
struct SoundWorld {
    current_patch: Uuid,
    anchors: Vec<Uuid>,
    candidates: Vec<Candidate>,
    exploration_radius: f32,
    novelty: f32,
    trajectory: Vec<WorldEvent>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct MusicState {
    bpm: f32,
    root: i32,
    scale: String,
    density: f32,
    tension: f32,
    movement: f32,
    novelty: f32,
    energy: f32,
}

impl Default for MusicState {
    fn default() -> Self {
        Self {
            bpm: 82.0,
            root: 36,
            scale: "minor".into(),
            density: 0.35,
            tension: 0.25,
            movement: 0.35,
            novelty: 0.18,
            energy: 0.5,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct HarmonyState {
    tonic: String,
    mode: String,
    chord_grid: Vec<String>,
    current_chord: String,
    key_mask: Vec<u8>,
    chord_mask: Vec<u8>,
    harmonic_rhythm_bars: f32,
}

impl Default for HarmonyState {
    fn default() -> Self {
        Self {
            tonic: "C".into(),
            mode: "minor".into(),
            chord_grid: vec!["Cm9".into(), "Abmaj7".into(), "Fm9".into(), "Gsus4".into()],
            current_chord: "Cm9".into(),
            key_mask: vec![0, 2, 3, 5, 7, 8, 10],
            chord_mask: vec![0, 3, 7, 10, 2],
            harmonic_rhythm_bars: 4.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct VisualState {
    geometry_scale: f32,
    deformation: f32,
    particle_density: f32,
    rotation_speed: f32,
    brightness: f32,
    complexity: f32,
    depth: f32,
    harmonic_position: [f32; 2],
}

impl Default for VisualState {
    fn default() -> Self {
        Self {
            geometry_scale: 0.6,
            deformation: 0.2,
            particle_density: 0.35,
            rotation_speed: 0.2,
            brightness: 0.45,
            complexity: 0.3,
            depth: 0.2,
            harmonic_position: [0.0, 0.0],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "event")]
enum WorldEvent {
    SessionStart { t: f32, seed: u64 },
    Nudge { t: f32, target: String, value: f32 },
    PatchAnchor { t: f32, patch: Uuid },
    Explore { t: f32, parent: Uuid },
}

struct SoundWorldApp {
    project: Project,
    corpus: CreativeCorpus,
    hardware: HardwareProbe,
    audio: Option<AudioEngine>,
    api_rx: Option<Receiver<ApiCommandRequest>>,
    api_state: Arc<Mutex<ApiStateSnapshot>>,
    mode: Mode,
    patch: Patch,
    patches: Vec<Patch>,
    world: SoundWorld,
    music: MusicState,
    harmony: HarmonyState,
    visual: VisualState,
    command: String,
    status: String,
    started: Instant,
    playing: bool,
    recording: bool,
    visuals_enabled: bool,
    last_note: Instant,
}

impl SoundWorldApp {
    fn new(
        hardware: HardwareProbe,
        audio: Option<AudioEngine>,
        api_rx: Option<Receiver<ApiCommandRequest>>,
        api_state: Arc<Mutex<ApiStateSnapshot>>,
    ) -> Self {
        let patch = Patch::init_bass();
        let patch_id = patch.id;
        let mut app = Self {
            project: Project::new("Default", hardware.sample_rate, 82.0),
            corpus: CreativeCorpus::default(),
            hardware,
            audio,
            api_rx,
            api_state,
            mode: Mode::World,
            patch: patch.clone(),
            patches: vec![patch],
            world: SoundWorld {
                current_patch: patch_id,
                anchors: vec![patch_id],
                candidates: vec![],
                exploration_radius: 0.35,
                novelty: 0.2,
                trajectory: vec![WorldEvent::SessionStart {
                    t: 0.0,
                    seed: 91832,
                }],
            },
            music: MusicState::default(),
            harmony: HarmonyState::default(),
            visual: VisualState::default(),
            command: String::new(),
            status: "Initialized local procedural SoundWorld".into(),
            started: Instant::now(),
            playing: false,
            recording: false,
            visuals_enabled: true,
            last_note: Instant::now(),
        };
        app.sync_audio_patch();
        app.register_current_patch_object("initial patch");
        app.generate_candidates();
        app.publish_api_state();
        app
    }

    fn publish_api_state(&self) {
        let snapshot = self.api_snapshot();
        if let Ok(mut state) = self.api_state.lock() {
            *state = snapshot;
        }
    }

    fn api_snapshot(&self) -> ApiStateSnapshot {
        ApiStateSnapshot {
            service: "soundworld".into(),
            playing: self.playing,
            recording: self.recording,
            visuals_enabled: self.visuals_enabled,
            mode: self.mode_label().into(),
            event_count: self.project.history.events.len(),
            candidate_count: self.world.candidates.len(),
            anchor_count: self.world.anchors.len(),
            patch: PatchSummary {
                id: self.patch.id,
                name: self.patch.name.clone(),
                sub_level: self.patch.sub_level,
                cutoff: self.patch.filter.cutoff,
                resonance: self.patch.filter.resonance,
                drive: self.patch.filter.drive,
                attack: self.patch.amp.attack,
                release: self.patch.amp.release,
                space: self.patch.effects.space,
            },
            music: self.music.clone(),
            harmony: self.harmony.clone(),
            visual: self.visual.clone(),
            affect: self.affect_state(),
        }
    }

    fn affect_state(&self) -> AffectState {
        let dissonance = interval_dissonance_for_chord(&self.harmony.chord_mask);
        let arousal = (0.35 * ((self.music.bpm - 40.0) / 110.0)
            + 0.25 * self.music.density
            + 0.2 * (1.0 - self.patch.amp.attack.clamp(0.0, 0.5) * 2.0)
            + 0.2 * self.visual.rotation_speed)
            .clamp(0.0, 1.0);
        let consonance = 1.0 - dissonance;
        let valence = (0.35 * consonance
            + 0.2 * (1.0 - self.patch.semantic.darkness)
            + 0.2 * self.patch.sub_level
            + 0.15 * (1.0 - self.music.tension)
            - 0.1 * self.patch.filter.drive)
            .clamp(0.0, 1.0);
        AffectState {
            valence,
            arousal,
            tension: self.music.tension,
            dissonance,
            voice_leading_distance: 0.0,
        }
    }

    fn mode_label(&self) -> &'static str {
        match self.mode {
            Mode::Synth => "Synth",
            Mode::World => "World",
            Mode::Track => "Track",
            Mode::Visual => "Visual",
            Mode::Session => "Session",
            Mode::Guide => "Guide",
        }
    }

    fn drain_api_commands(&mut self) {
        let Some(api_rx) = self.api_rx.as_ref().cloned() else {
            return;
        };
        for request in api_rx.try_iter() {
            let count = request.commands.len();
            for command in request.commands {
                self.apply_external_command(request.origin.clone(), command);
            }
            self.status = format!("API accepted {count} command(s)");
        }
    }

    fn apply_external_command(&mut self, origin: EventOrigin, command: Command) {
        match &command {
            Command::Transport(TransportCommand::Play) => self.playing = true,
            Command::Transport(TransportCommand::Stop) => self.playing = false,
            Command::Transport(TransportCommand::SetBpm(bpm)) => {
                self.music.bpm = (*bpm as f32).clamp(40.0, 150.0);
            }
            Command::Music(MusicCommand::Nudge {
                target,
                delta,
                beats: _,
            }) => self.nudge_without_event(target, *delta),
            Command::Music(MusicCommand::SetDensity(value)) => {
                self.music.density = clamp01(*value);
            }
            Command::Music(MusicCommand::SetTension(value)) => {
                self.music.tension = clamp01(*value);
            }
            Command::Music(MusicCommand::SetMovement(value)) => {
                self.music.movement = clamp01(*value);
            }
            Command::Visual(VisualCommand::SetScene { name }) => {
                self.visuals_enabled = name != "disabled";
                self.mode = if self.visuals_enabled {
                    Mode::Visual
                } else {
                    Mode::Track
                };
            }
            Command::World(WorldCommand::Explore { radius, .. }) => {
                self.world.exploration_radius = radius.clamp(0.05, 1.0);
                self.generate_candidates_only();
                self.world.trajectory.push(WorldEvent::Explore {
                    t: self.t(),
                    parent: self.patch.id,
                });
            }
            Command::Sound(SoundCommand::Mutate { radius, .. }) => {
                let mut rng = ChaCha8Rng::seed_from_u64(self.patch.seed + 991);
                mutate_patch(&mut self.patch, radius.clamp(0.0, 1.0), 0.25, &mut rng);
                self.sync_audio_patch();
            }
            Command::Sound(SoundCommand::Anchor { .. }) => self.anchor_current(),
            Command::Instrument(InstrumentCommand::NoteOn { midi, velocity, .. }) => {
                if let Some(audio) = &self.audio {
                    audio.note_on(*midi, velocity.clamp(0.0, 1.0));
                }
            }
            _ => {}
        }
        self.project.accept_command(origin, command);
    }

    fn register_current_patch_object(&mut self, context: &str) {
        let object = CreativeObject::patch(
            self.patch.name.clone(),
            self.patch.id,
            self.patch.tags.clone(),
        );
        let id = self.corpus.add_object_to_default_world(object);
        if context.contains("anchor") {
            self.corpus.anchor(id, context);
        }
    }

    fn sync_audio_patch(&self) {
        if let Some(audio) = &self.audio {
            audio.set_patch(&self.patch);
        }
    }

    fn t(&self) -> f32 {
        self.started.elapsed().as_secs_f32()
    }

    fn generate_candidates(&mut self) {
        self.generate_candidates_only();
        self.world.trajectory.push(WorldEvent::Explore {
            t: self.t(),
            parent: self.patch.id,
        });
        self.project.accept_command(
            EventOrigin::HumanUi,
            Command::World(WorldCommand::Explore {
                patch: crate::core::PatchId(self.patch.id),
                radius: self.world.exploration_radius,
            }),
        );
    }

    fn generate_candidates_only(&mut self) {
        let mut rng = ChaCha8Rng::seed_from_u64(self.patch.seed + self.patch.generation as u64 + 1);
        self.world.candidates.clear();
        for idx in 0..16 {
            let mut p = self.patch.clone();
            p.id = Uuid::new_v4();
            p.parent = Some(self.patch.id);
            p.generation += 1;
            p.seed = rng.gen();
            p.name = format!("{}-{}", self.patch.name, (b'a' + idx as u8) as char);
            mutate_patch(
                &mut p,
                self.world.exploration_radius,
                self.world.novelty,
                &mut rng,
            );
            let x = (p.filter.cutoff - self.patch.filter.cutoff) * 3.0 + rng.gen_range(-0.08..0.08);
            let y = (p.sub_level - 0.4) + (p.filter.drive * 0.5) + rng.gen_range(-0.08..0.08);
            let centroid = p.filter.cutoff * (1.0 + p.osc_a.wavetable_pos);
            let rms = (p.osc_a.level + p.osc_b.level + p.sub_level) / 3.0;
            self.world.candidates.push(Candidate {
                patch: p,
                pos: Vector2::new(x, y),
                rms,
                centroid,
            });
        }
    }

    fn anchor_current(&mut self) {
        if !self.world.anchors.contains(&self.patch.id) {
            self.world.anchors.push(self.patch.id);
        }
        if !self.patches.iter().any(|p| p.id == self.patch.id) {
            self.patches.push(self.patch.clone());
        }
        self.register_current_patch_object("anchor patch");
        self.world.trajectory.push(WorldEvent::PatchAnchor {
            t: self.t(),
            patch: self.patch.id,
        });
        self.project.accept_command(
            EventOrigin::HumanUi,
            Command::Sound(SoundCommand::Anchor {
                patch: crate::core::PatchId(self.patch.id),
            }),
        );
        self.save_patch().ok();
        self.status = "Anchored and saved patch".into();
    }

    fn save_patch(&self) -> Result<()> {
        let dir = data_dir().join("Default.soundworld/patches");
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", self.patch.name));
        fs::write(path, serde_json::to_string_pretty(&self.patch)?)?;
        Ok(())
    }

    fn save_project(&self) -> Result<()> {
        let root = data_dir().join("Default.soundworld");
        fs::create_dir_all(root.join("patches"))?;
        fs::create_dir_all(root.join("recordings"))?;
        let project = serde_json::json!({
            "mangy_project": self.project,
            "creative_corpus": self.corpus,
            "world": self.world,
            "music_state": self.music,
            "visual_state": self.visual,
            "current_patch": self.patch,
        });
        fs::write(
            root.join("project.json"),
            serde_json::to_string_pretty(&project)?,
        )?;
        Ok(())
    }

    fn execute_command(&mut self) {
        let cmd = self.command.trim().to_lowercase();
        if cmd.is_empty() {
            return;
        }
        let mut understood = true;
        match cmd.as_str() {
            "darker" | "more dark" => self.nudge("darkness", 0.15),
            "brighter" => self.nudge("darkness", -0.15),
            "more movement" | "faster" => self.nudge("movement", 0.15),
            "less movement" | "slower" => self.nudge("movement", -0.15),
            "more sparse" | "less dense" => self.nudge("density", -0.15),
            "more dense" => self.nudge("density", 0.15),
            "more strange" => self.nudge("novelty", 0.15),
            "less strange" => self.nudge("novelty", -0.15),
            "more tension" => self.nudge("tension", 0.15),
            "less tension" => self.nudge("tension", -0.15),
            "more space" => self.nudge("space", 0.15),
            "less space" => self.nudge("space", -0.15),
            "start ambient" | "start ambient track" | "generate ambient" => {
                self.playing = true;
                self.project.accept_command(
                    EventOrigin::HumanUi,
                    Command::Transport(TransportCommand::Play),
                );
            }
            "start ambient no visuals" | "start ambient track no visuals" => {
                self.playing = true;
                self.visuals_enabled = false;
                self.mode = Mode::Track;
                self.project.accept_command(
                    EventOrigin::HumanUi,
                    Command::Transport(TransportCommand::Play),
                );
                self.project.accept_command(
                    EventOrigin::HumanUi,
                    Command::Visual(VisualCommand::SetScene {
                        name: "disabled".into(),
                    }),
                );
            }
            "visuals" | "show visuals" | "enable visuals" => {
                self.visuals_enabled = true;
                self.mode = Mode::Visual;
                self.project.accept_command(
                    EventOrigin::HumanUi,
                    Command::Visual(VisualCommand::SetScene {
                        name: "harmonic_orbits_low".into(),
                    }),
                );
            }
            "no visuals" | "hide visuals" | "disable visuals" => {
                self.visuals_enabled = false;
                if self.mode == Mode::Visual {
                    self.mode = Mode::Track;
                }
                self.project.accept_command(
                    EventOrigin::HumanUi,
                    Command::Visual(VisualCommand::SetScene {
                        name: "disabled".into(),
                    }),
                );
            }
            "explore" => self.generate_candidates(),
            "anchor this" | "anchor" => self.anchor_current(),
            "create bass world" | "make bass world" => {
                self.corpus.ensure_default_world();
                self.status = "Bass World exists in the creative corpus".into();
            }
            "show me something" | "show me something i haven't noticed" => {
                self.status = self.suggest_unnoticed_relationship();
            }
            "new bass" => {
                self.patch = Patch::init_bass();
                self.sync_audio_patch();
                self.register_current_patch_object("new bass");
                self.project.accept_command(
                    EventOrigin::HumanUi,
                    Command::Sound(SoundCommand::Mutate {
                        patch: crate::core::PatchId(self.patch.id),
                        radius: 0.0,
                    }),
                );
            }
            _ => understood = false,
        }
        self.status = if understood {
            format!("Command: {}", cmd)
        } else {
            "Command not understood.".into()
        };
        self.command.clear();
    }

    fn nudge(&mut self, target: &str, delta: f32) {
        self.nudge_without_event(target, delta);
        self.world.trajectory.push(WorldEvent::Nudge {
            t: self.t(),
            target: target.into(),
            value: delta,
        });
        self.project.accept_command(
            EventOrigin::HumanUi,
            Command::Music(MusicCommand::Nudge {
                target: target.into(),
                delta,
                beats: 8.0,
            }),
        );
        self.sync_audio_patch();
    }

    fn nudge_without_event(&mut self, target: &str, delta: f32) {
        match target {
            "darkness" => {
                self.patch.semantic.darkness = clamp01(self.patch.semantic.darkness + delta);
                self.patch.filter.cutoff = clamp01(self.patch.filter.cutoff - delta * 0.65);
            }
            "movement" => {
                self.music.movement = clamp01(self.music.movement + delta);
                self.patch.semantic.movement = self.music.movement;
            }
            "density" => self.music.density = clamp01(self.music.density + delta),
            "novelty" => self.music.novelty = clamp01(self.music.novelty + delta),
            "tension" => self.music.tension = clamp01(self.music.tension + delta),
            "space" => {
                self.patch.effects.space = clamp01(self.patch.effects.space + delta);
                self.patch.semantic.space = self.patch.effects.space;
            }
            _ => {}
        }
        self.sync_audio_patch();
    }

    fn ambient_tick(&mut self) {
        if !self.playing
            || self.last_note.elapsed()
                < Duration::from_millis(
                    (60000.0 / self.music.bpm / (1.0 + self.music.density * 2.0)) as u64,
                )
        {
            return;
        }
        self.last_note = Instant::now();
        let scale = [0, 2, 3, 5, 7, 8, 10];
        let mut rng = ChaCha8Rng::seed_from_u64((self.t() * 8.0) as u64 + self.patch.seed);
        let degree = rng.gen_range(0..scale.len());
        let octave = if rng.gen_bool(self.music.energy as f64 * 0.35) {
            12
        } else {
            0
        };
        let midi = self.music.root + scale[degree] + octave;
        let velocity = 0.35 + self.music.energy * 0.55;
        if let Some(audio) = &self.audio {
            audio.note_on(midi as u8, velocity);
        }
    }
}

impl eframe::App for SoundWorldApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_api_commands();
        self.ambient_tick();
        if self.visuals_enabled {
            self.visual.geometry_scale =
                0.35 + self.music.energy * 0.5 + self.patch.sub_level * 0.2;
            self.visual.brightness = 0.2 + self.patch.filter.cutoff * 0.7;
            self.visual.deformation = self.music.tension * 0.7 + self.patch.filter.drive * 0.3;
            self.visual.particle_density = self.music.density;
            self.visual.rotation_speed = self.music.movement;
            self.visual.harmonic_position = [
                self.music.tension * 2.0 - 1.0,
                self.music.novelty * 2.0 - 1.0,
            ];
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("SOUNDWORLD");
                ui.label(format!("events {}", self.project.history.events.len()));
                for (label, mode) in [
                    ("SYNTH", Mode::Synth),
                    ("WORLD", Mode::World),
                    ("TRACK", Mode::Track),
                    ("VISUAL", Mode::Visual),
                    ("SESSION", Mode::Session),
                    ("GUIDE", Mode::Guide),
                ] {
                    if ui.selectable_label(self.mode == mode, label).clicked() {
                        self.mode = mode;
                    }
                }
                ui.separator();
                ui.label(format!(
                    "{} Hz | buffer {} | {}",
                    self.hardware.sample_rate,
                    self.hardware.recommended_buffer,
                    self.hardware.quality_profile
                ));
                ui.label(if self.visuals_enabled {
                    "visuals on"
                } else {
                    "visuals off"
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            Mode::Synth => self.ui_synth(ui),
            Mode::World => self.ui_world(ui),
            Mode::Track => self.ui_track(ui),
            Mode::Visual => self.ui_visual(ui),
            Mode::Session => self.ui_session(ui),
            Mode::Guide => self.ui_guide(ui),
        });

        egui::TopBottomPanel::bottom("bottom").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let response = ui.text_edit_singleline(&mut self.command);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.execute_command();
                }
                if ui.button("Explore").clicked() {
                    self.generate_candidates();
                }
                if ui.button("Anchor").clicked() {
                    self.anchor_current();
                }
                if ui
                    .button(if self.playing {
                        "Stop"
                    } else {
                        "Generate Ambient"
                    })
                    .clicked()
                {
                    self.playing = !self.playing;
                    self.project.accept_command(
                        EventOrigin::HumanUi,
                        Command::Transport(if self.playing {
                            TransportCommand::Play
                        } else {
                            TransportCommand::Stop
                        }),
                    );
                }
                if ui
                    .button(if self.recording {
                        "Stop Rec"
                    } else {
                        "Record WAV"
                    })
                    .clicked()
                {
                    self.recording = !self.recording;
                    if let Some(audio) = &self.audio {
                        if self.recording {
                            audio.start_recording().ok();
                        } else {
                            audio.stop_recording();
                        }
                    }
                }
                ui.label(&self.status);
            });
        });

        let repaint_ms = if self.playing
            || self.recording
            || (self.mode == Mode::Visual && self.visuals_enabled)
        {
            33
        } else {
            100
        };
        self.publish_api_state();
        ctx.request_repaint_after(Duration::from_millis(repaint_ms));
    }
}

impl SoundWorldApp {
    fn ui_synth(&mut self, ui: &mut egui::Ui) {
        ui.heading("Bass Synth");
        ui.label("Shape the current bass patch. Changes update the audio engine immediately.");
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Oscillator A");
                changed |= wave_combo(ui, &mut self.patch.osc_a.wave, "wave_a");
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.osc_a.level, 0.0..=1.0).text("level"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.osc_a.fine, -0.5..=0.5).text("fine"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.patch.osc_a.pulse_width, 0.05..=0.95)
                            .text("pulse width"),
                    )
                    .changed();
            });
            ui.vertical(|ui| {
                ui.label("Oscillator B");
                changed |= wave_combo(ui, &mut self.patch.osc_b.wave, "wave_b");
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.osc_b.level, 0.0..=1.0).text("level"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.osc_b.fine, -0.5..=0.5).text("fine"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.patch.osc_b.pulse_width, 0.05..=0.95)
                            .text("pulse width"),
                    )
                    .changed();
            });
            ui.vertical(|ui| {
                ui.label("Body");
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.sub_level, 0.0..=1.0).text("sub"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.noise_level, 0.0..=0.2).text("noise"))
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.patch.filter.cutoff, 0.02..=1.0).text("cutoff"),
                    )
                    .changed();
                changed |= ui
                    .add(
                        egui::Slider::new(&mut self.patch.filter.resonance, 0.0..=0.95)
                            .text("resonance"),
                    )
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.filter.drive, 0.0..=1.0).text("drive"))
                    .changed();
            });
            ui.vertical(|ui| {
                ui.label("Envelope / Space");
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.amp.attack, 0.001..=0.5).text("attack"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.amp.decay, 0.01..=1.0).text("decay"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.amp.sustain, 0.0..=1.0).text("sustain"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.amp.release, 0.01..=2.0).text("release"))
                    .changed();
                changed |= ui
                    .add(egui::Slider::new(&mut self.patch.effects.space, 0.0..=1.0).text("space"))
                    .changed();
            });
        });
        if changed {
            self.sync_audio_patch();
            self.project.accept_command(
                EventOrigin::HumanUi,
                Command::Sound(SoundCommand::SetParam {
                    instrument: self.project.instruments.instruments[0].id,
                    param: ParamId("mangy_synth.patch".into()),
                    value: 1.0,
                }),
            );
        }
        if ui.button("Audition C2").clicked() {
            if let Some(audio) = &self.audio {
                audio.note_on(36, 0.85);
            }
        }
        if ui.button("Apply to audio").clicked() {
            self.sync_audio_patch();
        }
        if ui.button("Save patch").clicked() {
            self.status = self
                .save_patch()
                .map(|_| "Saved patch".into())
                .unwrap_or_else(|e| e.to_string());
        }
    }

    fn ui_world(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sound World Map");
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 420.0),
            egui::Sense::click(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_stroke(
            rect,
            0.0,
            egui::Stroke::new(1.0_f32, egui::Color32::DARK_GRAY),
        );
        let center = rect.center();
        painter.line_segment(
            [
                egui::pos2(rect.left(), center.y),
                egui::pos2(rect.right(), center.y),
            ],
            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(50)),
        );
        painter.line_segment(
            [
                egui::pos2(center.x, rect.top()),
                egui::pos2(center.x, rect.bottom()),
            ],
            egui::Stroke::new(1.0_f32, egui::Color32::from_gray(50)),
        );
        painter.circle_stroke(
            center,
            7.0,
            egui::Stroke::new(2.0_f32, egui::Color32::WHITE),
        );
        let mut clicked_idx = None;
        for (idx, c) in self.world.candidates.iter().enumerate() {
            let p = egui::pos2(center.x + c.pos.x * 180.0, center.y - c.pos.y * 180.0);
            let gray = (95.0 + c.centroid * 110.0 + c.rms * 40.0).clamp(80.0, 235.0) as u8;
            painter.circle_stroke(
                p,
                6.0,
                egui::Stroke::new(1.5_f32, egui::Color32::from_gray(gray)),
            );
            if response.clicked()
                && response
                    .interact_pointer_pos()
                    .map(|m| m.distance(p) < 12.0)
                    .unwrap_or(false)
            {
                clicked_idx = Some(idx);
            }
        }
        if let Some(idx) = clicked_idx {
            self.patch = self.world.candidates[idx].patch.clone();
            self.sync_audio_patch();
            self.status = format!("Auditioning {}", self.patch.name);
            if let Some(audio) = &self.audio {
                audio.note_on(36, 0.8);
            }
        }
        ui.horizontal(|ui| {
            ui.add(
                egui::Slider::new(&mut self.world.exploration_radius, 0.05..=1.0).text("radius"),
            );
            ui.add(egui::Slider::new(&mut self.world.novelty, 0.0..=1.0).text("novelty"));
        });
    }

    fn ui_track(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ambient World");
        ui.add(egui::Slider::new(&mut self.music.bpm, 40.0..=150.0).text("bpm"));
        ui.add(egui::Slider::new(&mut self.music.density, 0.0..=1.0).text("density"));
        ui.add(egui::Slider::new(&mut self.music.tension, 0.0..=1.0).text("tension"));
        ui.add(egui::Slider::new(&mut self.music.movement, 0.0..=1.0).text("movement"));
        ui.add(egui::Slider::new(&mut self.music.novelty, 0.0..=1.0).text("novelty"));
        ui.add(egui::Slider::new(&mut self.music.energy, 0.0..=1.0).text("energy"));
        ui.label("Layer v1: deterministic bass/motif pulses using the current patch.");
        if ui.button("Commit music state to project graph").clicked() {
            self.project.accept_command(
                EventOrigin::HumanUi,
                Command::Transport(TransportCommand::SetBpm(self.music.bpm as f64)),
            );
            self.project.accept_command(
                EventOrigin::HumanUi,
                Command::Music(MusicCommand::SetDensity(self.music.density)),
            );
            self.project.accept_command(
                EventOrigin::HumanUi,
                Command::Music(MusicCommand::SetTension(self.music.tension)),
            );
            self.project.accept_command(
                EventOrigin::HumanUi,
                Command::Music(MusicCommand::SetMovement(self.music.movement)),
            );
            self.status = "Committed track controls to command/event graph".into();
        }
    }

    fn ui_visual(&mut self, ui: &mut egui::Ui) {
        ui.heading("Music-driven Procedural Visual");
        ui.horizontal(|ui| {
            if ui
                .button(if self.visuals_enabled {
                    "Disable visuals"
                } else {
                    "Enable visuals"
                })
                .clicked()
            {
                self.visuals_enabled = !self.visuals_enabled;
            }
            ui.label("Low-cost greyscale harmonic orbits.");
        });
        if !self.visuals_enabled {
            ui.label("Visual rendering is disabled. Audio and command/event recording continue.");
            return;
        }
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), 500.0),
            egui::Sense::hover(),
        );
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, egui::Color32::BLACK);
        let center = rect.center()
            + egui::vec2(
                self.visual.harmonic_position[0] * 80.0,
                self.visual.harmonic_position[1] * 60.0,
            );
        let time = if self.playing { self.t() } else { 0.0 };
        let rings = 5 + (self.visual.complexity * 4.0) as usize;
        for i in 0..rings {
            let r = 24.0 + i as f32 * 24.0 * self.visual.geometry_scale;
            let gray =
                (80.0 + self.visual.brightness * 150.0 + i as f32 * 7.0).clamp(70.0, 245.0) as u8;
            painter.circle_stroke(
                center,
                r,
                egui::Stroke::new(
                    1.0 + self.visual.deformation,
                    egui::Color32::from_gray(gray),
                ),
            );
            let angle = time * self.visual.rotation_speed + i as f32 * 0.9;
            let p = center + egui::vec2(angle.cos() * r, angle.sin() * r);
            painter.line_segment(
                [center, p],
                egui::Stroke::new(1.0_f32, egui::Color32::from_gray(100)),
            );
            painter.circle_filled(p, 2.0 + self.music.energy * 3.0, egui::Color32::WHITE);
        }
    }

    fn ui_session(&mut self, ui: &mut egui::Ui) {
        ui.heading("Session");
        ui.label(format!("Patches: {}", self.patches.len()));
        ui.label(format!("Anchors: {}", self.world.anchors.len()));
        ui.label(format!("Events: {}", self.world.trajectory.len()));
        ui.label(format!("Creative objects: {}", self.corpus.objects.len()));
        ui.label(format!("Corpus worlds: {}", self.corpus.worlds.len()));
        ui.label(format!(
            "Preference events: {}",
            self.corpus.preferences.len()
        ));
        ui.label(format!(
            "Project graph events: {}",
            self.project.history.events.len()
        ));
        ui.label(format!(
            "Transport: {:.1} bpm | playing {}",
            self.project.transport.bpm, self.project.transport.playing
        ));
        ui.label(format!("Visuals enabled: {}", self.visuals_enabled));
        if ui.button("Use harmonic_orbits_low scene").clicked() {
            self.project.accept_command(
                EventOrigin::HumanUi,
                Command::Visual(VisualCommand::SetScene {
                    name: "harmonic_orbits_low".into(),
                }),
            );
        }
        if ui.button("Save project").clicked() {
            self.status = self
                .save_project()
                .map(|_| "Saved project".into())
                .unwrap_or_else(|e| e.to_string());
        }
        ui.label(format!("Data: {}", data_dir().display()));
    }

    fn ui_guide(&mut self, ui: &mut egui::Ui) {
        ui.heading("Guide");
        ui.label("What this build is");
        ui.label("SoundWorld is a standalone Rust instrument: bass synth, mutation map, ambient generator, simple procedural visuals, patch JSON, and WAV recording.");
        ui.separator();
        ui.label("Make a sound");
        ui.label("1. Open SYNTH. Press Audition C2. Move cutoff, drive, sub, oscillator levels, and wave shapes.");
        ui.label("2. Press Save patch when it sounds useful.");
        ui.label("3. Open WORLD. Click candidate nodes to audition variations. Anchor the ones worth keeping.");
        ui.label("4. Open TRACK. Press Generate Ambient, then use density, movement, tension, novelty, and energy.");
        ui.label("5. Use the bottom command bar like a tiny chatbar. Current local commands: start ambient, start ambient no visuals, no visuals, visuals, darker, brighter, more movement, less dense, more strange, more tension, more space, explore, anchor.");
        ui.label("6. Press Record WAV to capture audio. Files land in ~/.local/share/soundworld/Default.soundworld/recordings.");
        ui.separator();
        ui.label("AI-native direction");
        ui.label("The command bar is intentionally shaped like a future chatbar. Today it uses a tiny local parser. Next, an LLM provider can translate free text into the same safe Command -> Event system.");
        ui.label("Example future prompt: start an ambient track with the anchored basses, no visuals, slowly get darker over 16 bars.");
        ui.label("Library/world direction: saved patches, samples, motifs, loops, modulations, harmony paths, and performance fragments become CreativeObjects. Worlds are queryable maps over that corpus.");
        ui.separator();
        ui.label("Other synths installed for DAWs");
        ui.label("Surge XT: standalone app plus VST3/LV2/CLAP plugins. Use it for polished bass sound design.");
        ui.label("Cardinal: VST3 modular synth plugin installed. Use Ardour's plugin scan, then add Cardinal/Surge XT to instrument tracks.");
        ui.label("Ardour + PipeWire + qpwgraph are installed for DAW routing.");
    }

    fn suggest_unnoticed_relationship(&self) -> String {
        let anchors = self
            .corpus
            .worlds
            .first()
            .map(|w| w.anchors.len())
            .unwrap_or(0);
        if self.corpus.objects.len() < 3 {
            "Not enough saved sounds yet. Create or anchor at least three patches first.".into()
        } else if anchors == 0 {
            "You have sounds in the corpus, but no anchored favorites yet. Anchor a few so Mangy can learn preference.".into()
        } else {
            format!(
                "Corpus has {} objects and {} anchors. Next build should compare sub weight, brightness, drive, and movement to find bridges.",
                self.corpus.objects.len(),
                anchors
            )
        }
    }
}

fn wave_combo(ui: &mut egui::Ui, wave: &mut Wave, id: &str) -> bool {
    let before = *wave;
    egui::ComboBox::from_id_source(id)
        .selected_text(format!("{:?}", wave))
        .show_ui(ui, |ui| {
            ui.selectable_value(wave, Wave::Sine, "Sine");
            ui.selectable_value(wave, Wave::Triangle, "Triangle");
            ui.selectable_value(wave, Wave::Saw, "Saw");
            ui.selectable_value(wave, Wave::Square, "Square");
            ui.selectable_value(wave, Wave::Wavetable, "Wavetable");
        });
    before != *wave
}

fn mutate_patch(p: &mut Patch, radius: f32, novelty: f32, rng: &mut ChaCha8Rng) {
    let amount = radius * (0.25 + novelty * 0.75);
    match rng.gen_range(0..6) {
        0 => {
            p.osc_a.wavetable_pos = clamp01(p.osc_a.wavetable_pos + rng.gen_range(-amount..amount));
            p.osc_b.level = clamp01(p.osc_b.level + rng.gen_range(-amount * 0.5..amount * 0.5));
        }
        1 => {
            p.filter.cutoff = clamp01(p.filter.cutoff + rng.gen_range(-amount..amount));
            p.filter.resonance =
                clamp01(p.filter.resonance + rng.gen_range(-amount * 0.4..amount * 0.4));
        }
        2 => {
            p.filter.drive = clamp01(p.filter.drive + rng.gen_range(-amount..amount));
            p.noise_level = clamp01(p.noise_level + rng.gen_range(-amount * 0.08..amount * 0.08));
        }
        3 => {
            p.amp.attack = (p.amp.attack + rng.gen_range(-0.02..0.08) * amount).clamp(0.001, 0.5);
            p.amp.release = (p.amp.release + rng.gen_range(-0.08..0.18) * amount).clamp(0.01, 2.0);
        }
        4 => {
            p.effects.space = clamp01(p.effects.space + rng.gen_range(-amount..amount));
            p.effects.width = clamp01(p.effects.width + rng.gen_range(-amount..amount));
        }
        _ => {
            p.sub_level = clamp01(p.sub_level + rng.gen_range(-amount..amount));
        }
    }
}

fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

fn interval_dissonance_for_chord(mask: &[u8]) -> f32 {
    if mask.len() < 2 {
        return 0.0;
    }
    let mut total = 0.0_f32;
    let mut pairs = 0.0_f32;
    for i in 0..mask.len() {
        for j in (i + 1)..mask.len() {
            let interval = ((mask[i] as i16 - mask[j] as i16).abs() % 12) as u8;
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

struct AtomicPatch {
    osc_a_wave: AtomicU32,
    osc_b_wave: AtomicU32,
    osc_a_level: AtomicU32,
    osc_b_level: AtomicU32,
    sub_level: AtomicU32,
    noise_level: AtomicU32,
    cutoff: AtomicU32,
    resonance: AtomicU32,
    drive: AtomicU32,
    space: AtomicU32,
}

impl AtomicPatch {
    fn new() -> Self {
        Self {
            osc_a_wave: AtomicU32::new(2),
            osc_b_wave: AtomicU32::new(3),
            osc_a_level: f32a(0.78),
            osc_b_level: f32a(0.28),
            sub_level: f32a(0.55),
            noise_level: f32a(0.015),
            cutoff: f32a(0.33),
            resonance: f32a(0.18),
            drive: f32a(0.24),
            space: f32a(0.18),
        }
    }
}

fn f32a(v: f32) -> AtomicU32 {
    AtomicU32::new(v.to_bits())
}
fn load_f32(a: &AtomicU32) -> f32 {
    f32::from_bits(a.load(Ordering::Relaxed))
}
fn store_f32(a: &AtomicU32, v: f32) {
    a.store(v.to_bits(), Ordering::Relaxed);
}

#[derive(Clone, Copy)]
struct NoteMsg {
    midi: u8,
    velocity: f32,
}

struct AudioEngine {
    patch: Arc<AtomicPatch>,
    note_tx: Sender<NoteMsg>,
    recording: Arc<AtomicBool>,
    _stream: cpal::Stream,
}

impl AudioEngine {
    fn start() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no output audio device")?;
        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0 as f32;
        let channels = config.channels() as usize;
        let patch = Arc::new(AtomicPatch::new());
        let (note_tx, note_rx) = bounded::<NoteMsg>(256);
        let recording = Arc::new(AtomicBool::new(false));
        let (writer_tx, writer_rx) = bounded::<(f32, f32)>(48_000);
        spawn_writer(writer_rx, recording.clone(), config.sample_rate().0);
        let mut synth = Synth::new(
            sample_rate,
            patch.clone(),
            note_rx,
            writer_tx.clone(),
            recording.clone(),
        );
        let err_fn = |err| eprintln!("audio stream error: {err}");
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config.into(),
                move |d: &mut [f32], _| synth.render(d, channels),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::I16 => device.build_output_stream(
                &config.into(),
                move |d: &mut [i16], _| synth.render_i16(d, channels),
                err_fn,
                None,
            )?,
            cpal::SampleFormat::U16 => device.build_output_stream(
                &config.into(),
                move |d: &mut [u16], _| synth.render_u16(d, channels),
                err_fn,
                None,
            )?,
            _ => anyhow::bail!("unsupported sample format"),
        };
        stream.play()?;
        Ok(Self {
            patch,
            note_tx,
            recording,
            _stream: stream,
        })
    }

    fn set_patch(&self, p: &Patch) {
        self.patch
            .osc_a_wave
            .store(wave_id(p.osc_a.wave), Ordering::Relaxed);
        self.patch
            .osc_b_wave
            .store(wave_id(p.osc_b.wave), Ordering::Relaxed);
        store_f32(&self.patch.osc_a_level, p.osc_a.level);
        store_f32(&self.patch.osc_b_level, p.osc_b.level);
        store_f32(&self.patch.sub_level, p.sub_level);
        store_f32(&self.patch.noise_level, p.noise_level);
        store_f32(&self.patch.cutoff, p.filter.cutoff);
        store_f32(&self.patch.resonance, p.filter.resonance);
        store_f32(&self.patch.drive, p.filter.drive);
        store_f32(&self.patch.space, p.effects.space);
    }

    fn note_on(&self, midi: u8, velocity: f32) {
        let _ = self.note_tx.try_send(NoteMsg { midi, velocity });
    }

    fn start_recording(&self) -> Result<()> {
        self.recording.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn stop_recording(&self) {
        self.recording.store(false, Ordering::Relaxed);
    }
}

fn wave_id(w: Wave) -> u32 {
    match w {
        Wave::Sine => 0,
        Wave::Triangle => 1,
        Wave::Saw => 2,
        Wave::Square => 3,
        Wave::Wavetable => 4,
    }
}

fn spawn_writer(rx: Receiver<(f32, f32)>, recording: Arc<AtomicBool>, sample_rate: u32) {
    spawn_writer_at(rx, recording, sample_rate, data_dir());
}

fn spawn_writer_at(
    rx: Receiver<(f32, f32)>,
    recording: Arc<AtomicBool>,
    sample_rate: u32,
    data_root: PathBuf,
) {
    std::thread::spawn(move || {
        let mut writer: Option<WavWriter<BufWriter<File>>> = None;
        let mut was_recording = false;
        loop {
            let is_recording = recording.load(Ordering::Relaxed);
            if is_recording && !was_recording {
                let dir = data_root.join("Default.soundworld/recordings");
                let _ = fs::create_dir_all(&dir);
                let path = dir.join(format!(
                    "session-{}.wav",
                    Local::now().format("%Y%m%d-%H%M%S")
                ));
                let spec = WavSpec {
                    channels: 2,
                    sample_rate,
                    bits_per_sample: 16,
                    sample_format: SampleFormat::Int,
                };
                writer = WavWriter::create(path, spec).ok();
            }
            if !is_recording && was_recording {
                if let Some(w) = writer.take() {
                    let _ = w.finalize();
                }
            }
            was_recording = is_recording;
            if let Ok((l, r)) = rx.recv_timeout(Duration::from_millis(20)) {
                if let Some(w) = writer.as_mut() {
                    let _ = w.write_sample((l.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
                    let _ = w.write_sample((r.clamp(-1.0, 1.0) * i16::MAX as f32) as i16);
                }
            }
        }
    });
}

struct Synth {
    sample_rate: f32,
    patch: Arc<AtomicPatch>,
    notes: Receiver<NoteMsg>,
    writer_tx: Sender<(f32, f32)>,
    recording: Arc<AtomicBool>,
    voices: [Voice; 6],
    lp: f32,
    bp: f32,
    rng: u32,
}

impl Synth {
    fn new(
        sample_rate: f32,
        patch: Arc<AtomicPatch>,
        notes: Receiver<NoteMsg>,
        writer_tx: Sender<(f32, f32)>,
        recording: Arc<AtomicBool>,
    ) -> Self {
        Self {
            sample_rate,
            patch,
            notes,
            writer_tx,
            recording,
            voices: [Voice::default(); 6],
            lp: 0.0,
            bp: 0.0,
            rng: 1,
        }
    }

    fn render(&mut self, out: &mut [f32], channels: usize) {
        for msg in self.notes.try_iter() {
            let voice_idx = self.voices.iter().position(|v| !v.active).unwrap_or(0);
            self.voices[voice_idx].start(msg.midi, msg.velocity, self.sample_rate);
        }
        for frame in out.chunks_mut(channels) {
            let sample = self.next_sample();
            for ch in frame.iter_mut() {
                *ch = sample;
            }
            if self.recording.load(Ordering::Relaxed) {
                let _ = self.writer_tx.try_send((sample, sample));
            }
        }
    }

    fn render_i16(&mut self, out: &mut [i16], channels: usize) {
        for frame in out.chunks_mut(channels) {
            let sample = self.next_frame_sample();
            let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            for ch in frame.iter_mut() {
                *ch = value;
            }
        }
    }

    fn render_u16(&mut self, out: &mut [u16], channels: usize) {
        for frame in out.chunks_mut(channels) {
            let sample = self.next_frame_sample();
            let value = ((sample.clamp(-1.0, 1.0) * 0.5 + 0.5) * u16::MAX as f32) as u16;
            for ch in frame.iter_mut() {
                *ch = value;
            }
        }
    }

    fn next_frame_sample(&mut self) -> f32 {
        for msg in self.notes.try_iter() {
            let voice_idx = self.voices.iter().position(|v| !v.active).unwrap_or(0);
            self.voices[voice_idx].start(msg.midi, msg.velocity, self.sample_rate);
        }
        let sample = self.next_sample();
        if self.recording.load(Ordering::Relaxed) {
            let _ = self.writer_tx.try_send((sample, sample));
        }
        sample
    }

    fn next_sample(&mut self) -> f32 {
        let mut s = 0.0;
        let wave_a = self.patch.osc_a_wave.load(Ordering::Relaxed);
        let wave_b = self.patch.osc_b_wave.load(Ordering::Relaxed);
        let a = load_f32(&self.patch.osc_a_level);
        let b = load_f32(&self.patch.osc_b_level);
        let sub = load_f32(&self.patch.sub_level);
        for v in &mut self.voices {
            if v.active {
                s += v.next(wave_a, a, wave_b, b, sub);
            }
        }
        self.rng = self.rng.wrapping_mul(1664525).wrapping_add(1013904223);
        let noise = ((self.rng >> 9) as f32 / (1u32 << 23) as f32 - 1.0)
            * load_f32(&self.patch.noise_level);
        s += noise;
        s = svf_lowpass(
            s,
            &mut self.lp,
            &mut self.bp,
            load_f32(&self.patch.cutoff),
            load_f32(&self.patch.resonance),
        );
        let drive = 1.0 + load_f32(&self.patch.drive) * 8.0;
        (s * drive).tanh() * 0.22
    }
}

#[derive(Clone, Copy)]
struct Voice {
    active: bool,
    freq: f32,
    velocity: f32,
    phase_a: f32,
    phase_b: f32,
    phase_sub: f32,
    age: f32,
    sample_rate: f32,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            active: false,
            freq: 55.0,
            velocity: 0.0,
            phase_a: 0.0,
            phase_b: 0.0,
            phase_sub: 0.0,
            age: 0.0,
            sample_rate: 48_000.0,
        }
    }
}

impl Voice {
    fn start(&mut self, midi: u8, velocity: f32, sample_rate: f32) {
        self.active = true;
        self.freq = 440.0 * 2.0_f32.powf((midi as f32 - 69.0) / 12.0);
        self.velocity = velocity;
        self.phase_a = 0.0;
        self.phase_b = 0.0;
        self.phase_sub = 0.0;
        self.age = 0.0;
        self.sample_rate = sample_rate;
    }

    fn next(&mut self, wave_a: u32, a: f32, wave_b: u32, b: f32, sub: f32) -> f32 {
        self.age += 1.0 / self.sample_rate;
        if self.age > 2.2 {
            self.active = false;
            return 0.0;
        }
        let env = if self.age < 0.01 {
            self.age / 0.01
        } else {
            (1.0 - (self.age - 0.01) / 2.2).clamp(0.0, 1.0)
        };
        let sa = osc(wave_a, self.phase_a);
        let sb = osc(wave_b, self.phase_b);
        let ss = osc(0, self.phase_sub);
        self.phase_a = (self.phase_a + self.freq / self.sample_rate) % 1.0;
        self.phase_b = (self.phase_b + self.freq * 0.997 / self.sample_rate) % 1.0;
        self.phase_sub = (self.phase_sub + self.freq * 0.5 / self.sample_rate) % 1.0;
        (sa * a + sb * b + ss * sub) * env * self.velocity
    }
}

fn osc(wave: u32, phase: f32) -> f32 {
    match wave {
        0 => (phase * TAU).sin(),
        1 => 1.0 - 4.0 * (phase - 0.5).abs(),
        2 => 2.0 * phase - 1.0,
        3 => {
            if phase < 0.5 {
                1.0
            } else {
                -1.0
            }
        }
        _ => {
            (phase * TAU).sin() * 0.7
                + (phase * TAU * 2.0).sin() * 0.2
                + (phase * TAU * 3.0).sin() * 0.1
        }
    }
}

fn svf_lowpass(input: f32, lp: &mut f32, bp: &mut f32, cutoff: f32, resonance: f32) -> f32 {
    let f = (cutoff.clamp(0.001, 0.99) * 0.22).clamp(0.001, 0.22);
    let q = 1.0 - resonance.clamp(0.0, 0.95);
    let hp = input - *lp - q * *bp;
    *bp += f * hp;
    *lp += f * *bp;
    *lp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossbeam_channel::bounded;
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use std::time::Duration;
    use std::{fs, thread};

    fn free_bind_addr() -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.local_addr().unwrap().to_string()
    }

    fn http_request(addr: &str, request: &str) -> String {
        let mut stream = TcpStream::connect(addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        response
    }

    #[test]
    fn project_accepts_transport_command_as_event() {
        let mut project = Project::new("test", 48_000, 82.0);
        assert!(!project.transport.playing);
        assert_eq!(project.history.events.len(), 0);

        project.accept_command(
            EventOrigin::HumanUi,
            Command::Transport(TransportCommand::Play),
        );

        assert!(project.transport.playing);
        assert_eq!(project.history.events.len(), 1);
    }

    #[test]
    fn creative_corpus_records_anchor_preference() {
        let mut corpus = CreativeCorpus::default();
        let object = CreativeObject::patch("test bass", Uuid::new_v4(), vec!["bass".into()]);
        let id = corpus.add_object_to_default_world(object);
        corpus.anchor(id, "unit test anchor");

        assert_eq!(corpus.objects.len(), 1);
        assert_eq!(corpus.worlds.len(), 1);
        assert_eq!(corpus.worlds[0].anchors, vec![id]);
        assert_eq!(corpus.preferences.len(), 1);
    }

    #[test]
    fn oscillator_and_filter_produce_finite_signal() {
        for wave in 0..=4 {
            let sample = osc(wave, 0.25);
            assert!(sample.is_finite());
            assert!((-1.25..=1.25).contains(&sample));
        }

        let mut lp = 0.0;
        let mut bp = 0.0;
        let filtered = svf_lowpass(0.5, &mut lp, &mut bp, 0.4, 0.2);
        assert!(filtered.is_finite());
    }

    #[test]
    fn synth_generates_nonzero_audio_after_note() {
        let patch = Arc::new(AtomicPatch::new());
        let (note_tx, note_rx) = bounded::<NoteMsg>(8);
        let (writer_tx, _writer_rx) = bounded::<(f32, f32)>(8);
        let recording = Arc::new(AtomicBool::new(false));
        let mut synth = Synth::new(48_000.0, patch, note_rx, writer_tx, recording.clone());

        note_tx
            .send(NoteMsg {
                midi: 36,
                velocity: 0.9,
            })
            .unwrap();

        let mut peak = 0.0_f32;
        for _ in 0..512 {
            peak = peak.max(synth.next_frame_sample().abs());
        }

        assert!(peak > 0.0001, "expected audible nonzero synth output");
        assert!(!recording.load(Ordering::Relaxed));
    }

    #[test]
    fn api_health_endpoint_responds() {
        let addr = free_bind_addr();
        let state = Arc::new(Mutex::new(ApiStateSnapshot::default()));
        let _rx = start_api_server(&addr, state).unwrap();

        let response = http_request(&addr, "GET /health HTTP/1.1\r\nHost: localhost\r\n\r\n");

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"ok\":true"));
    }

    #[test]
    fn api_command_endpoint_delivers_typed_commands() {
        let addr = free_bind_addr();
        let state = Arc::new(Mutex::new(ApiStateSnapshot::default()));
        let rx = start_api_server(&addr, state).unwrap();
        let request = ApiCommandRequest {
            origin: EventOrigin::Ai,
            commands: vec![
                Command::Transport(TransportCommand::Play),
                Command::Music(MusicCommand::SetDensity(0.25)),
            ],
        };
        let body = serde_json::to_string(&request).unwrap();
        let wire = format!(
            "POST /commands HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let response = http_request(&addr, &wire);
        let delivered = rx.recv_timeout(Duration::from_secs(2)).unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"accepted\":2"));
        assert_eq!(delivered.commands.len(), 2);
    }

    #[test]
    fn api_state_endpoint_returns_harmony_snapshot() {
        let addr = free_bind_addr();
        let state = Arc::new(Mutex::new(ApiStateSnapshot {
            playing: true,
            harmony: HarmonyState::default(),
            affect: AffectState {
                tension: 0.42,
                ..Default::default()
            },
            ..Default::default()
        }));
        let _rx = start_api_server(&addr, state).unwrap();

        let response = http_request(&addr, "GET /state HTTP/1.1\r\nHost: localhost\r\n\r\n");

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"playing\":true"));
        assert!(response.contains("\"chord_grid\""));
        let body = response.split("\r\n\r\n").nth(1).unwrap();
        let json: serde_json::Value = serde_json::from_str(body).unwrap();
        assert!((json["affect"]["tension"].as_f64().unwrap() - 0.42).abs() < 0.001);
    }

    #[test]
    fn api_macro_endpoint_converts_agent_words_to_commands() {
        let addr = free_bind_addr();
        let state = Arc::new(Mutex::new(ApiStateSnapshot::default()));
        let rx = start_api_server(&addr, state).unwrap();
        let body = r#"{"origin":"Ai","intent":"dark ambient","macros":["ambient","dark","wide","play","nonsense"]}"#;
        let wire = format!(
            "POST /macro HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );

        let response = http_request(&addr, &wire);
        let delivered = rx.recv_timeout(Duration::from_secs(2)).unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("\"accepted\""));
        assert!(response.contains("nonsense"));
        assert!(delivered.commands.len() >= 5);
    }

    #[test]
    fn app_applies_api_commands_to_product_state() {
        let hardware = HardwareProbe {
            cpu_threads: 2,
            ram_mb: 4096,
            audio_device: "test".into(),
            sample_rate: 48_000,
            recommended_buffer: 512,
            opengl_version: "test".into(),
            quality_profile: "low".into(),
        };
        let api_state = Arc::new(Mutex::new(ApiStateSnapshot::default()));
        let mut app = SoundWorldApp::new(hardware, None, None, api_state.clone());
        let initial_candidates = app.world.candidates.len();

        app.apply_external_command(EventOrigin::Ai, Command::Transport(TransportCommand::Play));
        app.apply_external_command(
            EventOrigin::Ai,
            Command::Music(MusicCommand::SetDensity(0.2)),
        );
        app.apply_external_command(
            EventOrigin::Ai,
            Command::Visual(VisualCommand::SetScene {
                name: "disabled".into(),
            }),
        );
        app.apply_external_command(
            EventOrigin::Ai,
            Command::World(WorldCommand::Explore {
                patch: crate::core::PatchId(app.patch.id),
                radius: 0.45,
            }),
        );

        assert!(app.playing);
        assert_eq!(app.music.density, 0.2);
        assert!(!app.visuals_enabled);
        assert_eq!(app.mode, Mode::Track);
        assert_eq!(app.world.candidates.len(), initial_candidates);
        assert!(matches!(
            app.project.history.events.last().map(|event| &event.origin),
            Some(EventOrigin::Ai)
        ));
        app.publish_api_state();
        assert_eq!(api_state.lock().unwrap().music.density, 0.2);
    }

    #[test]
    fn records_synth_run_to_nonzero_wav() {
        let data_root = std::env::temp_dir().join(format!("soundworld-test-{}", Uuid::new_v4()));
        let patch = Arc::new(AtomicPatch::new());
        let (note_tx, note_rx) = bounded::<NoteMsg>(8);
        let (writer_tx, writer_rx) = bounded::<(f32, f32)>(48_000);
        let recording = Arc::new(AtomicBool::new(false));
        spawn_writer_at(writer_rx, recording.clone(), 48_000, data_root.clone());
        let mut synth = Synth::new(48_000.0, patch, note_rx, writer_tx, recording.clone());

        recording.store(true, Ordering::Relaxed);
        note_tx
            .send(NoteMsg {
                midi: 36,
                velocity: 0.9,
            })
            .unwrap();
        for _ in 0..4096 {
            let sample = synth.next_frame_sample();
            synth.writer_tx.send((sample, sample)).unwrap();
        }
        recording.store(false, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(120));

        let recordings = data_root.join("Default.soundworld/recordings");
        let wav_path = fs::read_dir(&recordings)
            .unwrap()
            .filter_map(|entry| entry.ok().map(|entry| entry.path()))
            .find(|path| path.extension().is_some_and(|ext| ext == "wav"))
            .expect("expected a recorded wav file");
        let mut reader = hound::WavReader::open(&wav_path).unwrap();
        let spec = reader.spec();
        let samples: Vec<i16> = reader.samples::<i16>().map(|s| s.unwrap()).collect();
        let peak = samples.iter().map(|s| s.abs()).max().unwrap_or(0);

        assert_eq!(spec.channels, 2);
        assert_eq!(spec.sample_rate, 48_000);
        assert!(samples.len() >= 4096, "expected recorded stereo samples");
        assert!(peak > 0, "expected nonzero recorded audio");

        let _ = fs::remove_dir_all(data_root);
    }
}
