# SoundWorld Testing Notes

Date: 2026-08-15

This is a prototype validation pass, not a claim that SoundWorld is a finished DAW or plugin host. The current milestone is a standalone GUI/audio instrument with documentation for using Surge XT and Cardinal alongside a real DAW.

## Validation Summary

| Area | Status | Notes |
| --- | --- | --- |
| Rust formatting | PASS | `cargo fmt` completed cleanly. |
| Unit tests | PASS | 16 tests passed. |
| Crate check | PASS | `cargo check` completed cleanly. |
| Optimized build | PASS | `cargo build --release` completed cleanly. |
| GUI launch | PASS | Installed release binary launched on `DISPLAY=:0` and stayed alive until timeout killed it. |
| Local API | PASS | `GET /health`, `GET /state`, `POST /commands`, `POST /macro`, and `POST /llm` are covered by unit tests. |
| API drives app state | PASS | Test applies API-style commands and verifies playback, density, visuals, exploration, and AI-origin event logging. |
| Live API drives GUI | PASS | Running GUI returned state, accepted macro/command/LLM batches, showed changed state after application, and stayed alive until the smoke-test timeout. |
| Synth makes signal | PASS | Headless DSP test generated nonzero finite samples after a MIDI note. |
| Visuals render path | PASS/WARN | GUI render loop smoke-tested. No crash. No screenshot or pixel-level validation was performed in this pass. |
| Physical speaker output | WARN | DSP output is validated, but nobody listened to the speakers during this automated pass. |
| WAV recording | PASS | `records_synth_run_to_nonzero_wav` writes generated synth audio through the WAV writer, reopens the file, and verifies stereo 48 kHz nonzero samples. |
| Surge XT/Cardinal in DAW | NOT TESTED | Existing documentation explains using them in Ardour/REAPER. This pass did not rescan DAW plugin paths or launch a DAW. |

## Commands Run

Run these from `soundworld/`.

```bash
cargo fmt
CARGO_TARGET_DIR=/root/mangy-cargo-target /root/.cargo/bin/cargo test
CARGO_TARGET_DIR=/root/mangy-cargo-target /root/.cargo/bin/cargo check
CARGO_TARGET_DIR=/root/mangy-cargo-target /root/.cargo/bin/cargo build --release
```

The release binary was installed with:

```bash
cp /root/mangy-cargo-target/release/soundworld /home/box/.local/bin/soundworld
chown box:box /home/box/.local/bin/soundworld
```

The GUI smoke test was:

```bash
timeout 8 runuser -u box -- env \
  DISPLAY=:0 \
  XAUTHORITY=/home/box/.Xauthority \
  DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus \
  XDG_RUNTIME_DIR=/run/user/1000 \
  /home/box/.local/bin/soundworld
```

Exit code `124` is expected for this smoke test because `timeout` killed the still-running GUI process after 8 seconds. There was no crash output.

The live API smoke test used:

```bash
curl -s http://127.0.0.1:3769/health
```

Result:

```json
{"ok":true,"service":"soundworld"}
```

Then:

```bash
curl -s http://127.0.0.1:3769/commands \
  -H 'Content-Type: application/json' \
  -d '{"origin":"Ai","commands":[{"Transport":"Play"},{"Music":{"SetDensity":0.25}},{"Music":{"SetMovement":0.7}},{"Music":{"SetTension":0.55}},{"Visual":{"SetScene":{"name":"disabled"}}},{"World":{"Explore":{"patch":"00000000-0000-0000-0000-000000000000","radius":0.45}}},{"Instrument":{"NoteOn":{"instrument":"00000000-0000-0000-0000-000000000000","midi":36,"velocity":0.8}}}]}'
```

Result:

```json
{"accepted":7,"rejected":[]}
```

The GUI process stayed alive until the 20 second timeout killed it.

The live macro/state smoke test used:

```bash
curl -s http://127.0.0.1:3769/state
```

Before macro:

```text
playing=false
bpm=82.0
density=0.35
movement=0.35
tension=0.25
```

Then:

```bash
curl -s http://127.0.0.1:3769/macro \
  -H 'Content-Type: application/json' \
  -d '{"origin":"Ai","intent":"dark ambient low arousal","macros":["ambient","dark","wide","calm","play"]}'
```

Result:

```json
{"accepted":12,"rejected":[]}
```

After macro:

```text
playing=true
bpm=62.0
density=0.16
movement=0.14
tension=0.18
cutoff=0.20
space=0.78
event_count=13
```

The GUI process stayed alive until the 35 second timeout killed it.

The live OpenCode-style `/llm` smoke test used:

```bash
curl -s http://127.0.0.1:3769/llm \
  -H 'Content-Type: application/json' \
  -d '{"provider":"opencode-go","text":"start dark ambient low arousal wide drone with visuals","apply":true}'
```

Result:

```json
{"accepted":13,"applied":true,"provider":"opencode-go","rejected":[]}
```

After `/llm` state included:

```text
playing=true
mode=Visual
bpm=62.0
density=0.16
movement=0.14
tension=0.18
cutoff=0.20
space=0.78
event_count=14
```

The GUI process stayed alive until the 25 second timeout killed it.

## Unit Test Coverage Added

The current red-team tests in `src/main.rs` cover:

- `project_accepts_transport_command_as_event`: verifies transport commands mutate project state and create history events.
- `creative_corpus_records_anchor_preference`: verifies the creative corpus stores objects, anchors, worlds, and user preferences.
- `oscillator_and_filter_produce_finite_signal`: verifies oscillator waves and the low-pass filter produce finite bounded samples.
- `synth_generates_nonzero_audio_after_note`: sends a MIDI note into the synth and verifies generated output is nonzero.
- `api_health_endpoint_responds`: verifies the localhost API health endpoint responds.
- `api_command_endpoint_delivers_typed_commands`: verifies JSON commands travel through the HTTP server into the app command channel.
- `api_state_endpoint_returns_harmony_snapshot`: verifies the API exposes harmony, affect, and transport state.
- `api_macro_endpoint_converts_agent_words_to_commands`: verifies macro words become typed commands and unknown macro words are reported.
- `text_to_macros_extracts_ambient_intent`: verifies free text maps to deterministic musical macros.
- `api_llm_endpoint_accepts_opencode_style_text`: verifies `POST /llm` accepts OpenCode-style provider text and delivers commands when `apply=true`.
- `app_applies_api_commands_to_product_state`: verifies API-style commands affect real app state and log AI-origin events.
- `records_synth_run_to_nonzero_wav`: verifies generated synth audio is written to a readable stereo WAV file with nonzero samples.
- `pattern_queries_values_by_phase`: verifies `Pattern<T>` can be queried by beat phase.
- `pattern_handles_events_wrapping_period_boundary`: verifies cyclic pattern events work when they cross the period boundary.
- `voice_leading_distance_prefers_small_motion`: verifies the native voice-leading metric ranks smaller motion lower.
- `harmony_explorer_scores_bridge_with_constraints`: verifies the first `HarmonyExplorer` returns a measured bridge candidate with constraint-aware scoring.

## Red-Team Findings

1. Building inside `/tmp/mangycunt-soundworld/soundworld/target` can fail on this machine because `/tmp` is a small tmpfs.
2. Use `CARGO_TARGET_DIR=/root/mangy-cargo-target` for repeatable local builds on this system.
3. Current audio validation proves the synth DSP path emits a signal and the WAV writer can save nonzero generated audio. It does not prove ALSA/PipeWire speaker routing is audible.
4. Current visual validation proves the app opens and keeps a GUI render loop alive. It does not prove every visual mode is correctly framed on every viewport.
5. SoundWorld should still be treated as a standalone instrument/runtime, not a replacement DAW. Surge XT and Cardinal should be used in Ardour/REAPER for now.
6. API tests require permission to bind/connect localhost. In the restricted sandbox they fail with `Operation not permitted`; outside that sandbox they pass.
7. `POST /macro` reports rejected unknown macro words. `POST /commands` still accepts well-formed typed commands directly, so a future security pass should add an explicit allowlist if untrusted clients can reach it.
8. Red-team found that the first `Pattern<T>` implementation did not handle events spanning the cycle boundary. This is fixed and covered by `pattern_handles_events_wrapping_period_boundary`.
9. The current `HarmonyExplorer` is intentionally minimal: it scores a direct bridge, not multi-step path search. The `steps` field exists but is not yet used for intermediate-node generation.
10. `chord_roughness` is still a pitch-class approximation. It is not a Sethares/Plomp-Levelt partials-based model yet.
11. `POST /llm` is an OpenCode-compatible local endpoint, not a live external LLM call. It uses deterministic intent parsing so an external OpenCode/Go service can call SoundWorld safely today.

## Next Validation Items

- Add screenshot or pixel-read validation for the visual modes.
- Launch from the desktop `.desktop` entry, not just the binary path.
- Validate real audio device output through PipeWire/ALSA with a short audible tone test.
- Validate Ardour or REAPER can see Surge XT and Cardinal from the documented plugin paths.
- Add multi-step bridge search tests once `HarmonyExplorer` uses intermediate nodes.
