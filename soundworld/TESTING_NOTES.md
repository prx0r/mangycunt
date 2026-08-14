# SoundWorld Testing Notes

Date: 2026-08-15

This is a prototype validation pass, not a claim that SoundWorld is a finished DAW or plugin host. The current milestone is a standalone GUI/audio instrument with documentation for using Surge XT and Cardinal alongside a real DAW.

## Validation Summary

| Area | Status | Notes |
| --- | --- | --- |
| Rust formatting | PASS | `cargo fmt` completed cleanly. |
| Unit tests | PASS | 7 tests passed. |
| Crate check | PASS | `cargo check` completed cleanly. |
| Optimized build | PASS | `cargo build --release` completed cleanly. |
| GUI launch | PASS | Installed release binary launched on `DISPLAY=:0` and stayed alive until timeout killed it. |
| Local API | PASS | `GET /health` and `POST /commands` are covered by unit tests. |
| API drives app state | PASS | Test applies API-style commands and verifies playback, density, visuals, exploration, and AI-origin event logging. |
| Live API drives GUI | PASS | Running GUI returned API health OK, accepted a 7-command AI batch, and stayed alive until the smoke-test timeout. |
| Synth makes signal | PASS | Headless DSP test generated nonzero finite samples after a MIDI note. |
| Visuals render path | PASS/WARN | GUI render loop smoke-tested. No crash. No screenshot or pixel-level validation was performed in this pass. |
| Physical speaker output | WARN | DSP output is validated, but nobody listened to the speakers during this automated pass. |
| WAV recording | NOT TESTED | WAV writer exists in the app, but recording start/stop and file playback were not validated end to end here. |
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

## Unit Test Coverage Added

The current red-team tests in `src/main.rs` cover:

- `project_accepts_transport_command_as_event`: verifies transport commands mutate project state and create history events.
- `creative_corpus_records_anchor_preference`: verifies the creative corpus stores objects, anchors, worlds, and user preferences.
- `oscillator_and_filter_produce_finite_signal`: verifies oscillator waves and the low-pass filter produce finite bounded samples.
- `synth_generates_nonzero_audio_after_note`: sends a MIDI note into the synth and verifies generated output is nonzero.
- `api_health_endpoint_responds`: verifies the localhost API health endpoint responds.
- `api_command_endpoint_delivers_typed_commands`: verifies JSON commands travel through the HTTP server into the app command channel.
- `app_applies_api_commands_to_product_state`: verifies API-style commands affect real app state and log AI-origin events.

## Red-Team Findings

1. Building inside `/tmp/mangycunt-soundworld/soundworld/target` can fail on this machine because `/tmp` is a small tmpfs.
2. Use `CARGO_TARGET_DIR=/root/mangy-cargo-target` for repeatable local builds on this system.
3. Current audio validation proves the synth DSP path emits a signal. It does not prove ALSA/PipeWire speaker routing is audible.
4. Current visual validation proves the app opens and keeps a GUI render loop alive. It does not prove every visual mode is correctly framed on every viewport.
5. SoundWorld should still be treated as a standalone instrument/runtime, not a replacement DAW. Surge XT and Cardinal should be used in Ardour/REAPER for now.
6. API tests require permission to bind/connect localhost. In the restricted sandbox they fail with `Operation not permitted`; outside that sandbox they pass.
7. The API currently accepts well-formed typed commands. It does not yet perform a separate allowlist rejection pass, so the next security pass should reject commands outside the intended LLM tool surface before they hit the app.

## Next Validation Items

- Add an end-to-end WAV recording test that starts recording, writes samples, stops, and verifies a readable `.wav` file with nonzero audio.
- Add screenshot or pixel-read validation for the visual modes.
- Launch from the desktop `.desktop` entry, not just the binary path.
- Validate real audio device output through PipeWire/ALSA with a short audible tone test.
- Validate Ardour or REAPER can see Surge XT and Cardinal from the documented plugin paths.
