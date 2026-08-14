# API / LLM Control Surface

SoundWorld can be made controllable by an LLM API without forcing the LLM into the GUI or realtime audio thread.

The right model is:

```text
GUI command bar
local HTTP endpoint
local process stdin/stdout
OpenCode/OpenRouter/OpenAI adapter
        ↓
validated SoundWorld Command values
        ↓
Project.accept_command(...)
        ↓
Event history + synth/world/visual state
```

## Current State

Implemented now:

- typed `Command` enums
- immutable `Event` history
- `Project.accept_command(...)`
- local command bar parser
- AI planner data structures
- localhost HTTP server on `127.0.0.1:3769`
- `GET /health`
- `GET /state`
- `POST /commands`
- `POST /macro`
- `POST /llm`
- app-side command application for transport, music density/tension/movement, visual scene, world exploration, patch mutation, anchoring, and note-on audition
- shared harmony/affect/music/visual snapshot for agent inspection
- tests for API health, state, macro conversion, LLM text parsing, command delivery, and app-state changes

Not implemented yet:

- live LLM provider calls
- chat review UI
- authentication beyond localhost-only binding
- direct DAW/plugin hosting

So the answer is: yes, this is possible, and the first local control API is now implemented.

## Recommended First API

Start with a localhost-only command API, not a full DAW scripting engine.

Example endpoint:

```text
POST http://127.0.0.1:3769/commands
```

Example request:

```json
{
  "origin": "Ai",
  "commands": [
    { "Transport": "Play" },
    { "Visual": { "SetScene": { "name": "disabled" } } },
    { "Music": { "SetDensity": 0.35 } }
  ]
}
```

Example response:

```json
{
  "accepted": 3,
  "rejected": []
}
```

Health check:

```bash
curl -s http://127.0.0.1:3769/health
```

Inspect current state:

```bash
curl -s http://127.0.0.1:3769/state
```

Drive the running app:

```bash
curl -s http://127.0.0.1:3769/commands \
  -H 'Content-Type: application/json' \
  -d '{"origin":"Ai","commands":[{"Transport":"Play"},{"Music":{"SetDensity":0.25}},{"Music":{"SetMovement":0.7}},{"Visual":{"SetScene":{"name":"disabled"}}}]}'
```

Use macro words:

```bash
curl -s http://127.0.0.1:3769/macro \
  -H 'Content-Type: application/json' \
  -d '{"origin":"Ai","intent":"dark ambient","macros":["ambient","dark","wide","play"]}'
```

OpenCode/local LLM adapter:

```bash
curl -s http://127.0.0.1:3769/llm \
  -H 'Content-Type: application/json' \
  -d '{"provider":"opencode-go","text":"start dark ambient low arousal wide drone","apply":true}'
```

This endpoint is intentionally local and deterministic for now. It does not call a network model from the audio app. OpenCode, a Go service, or any other agent can call it directly, or translate richer plans into `/commands`.

## LLM Provider Shape

Do not let the model send shell commands or edit files. The model should return an `AiPlan`:

```json
{
  "summary": "Start a sparse ambient section with visuals off.",
  "commands": [
    { "Transport": "Play" },
    { "Visual": { "SetScene": { "name": "disabled" } } }
  ],
  "confidence": 0.86
}
```

SoundWorld validates the plan, then applies only allowed commands.

## Safe Command Allowlist

Good first commands:

- play / stop
- set or ramp synth parameters
- start ambient
- set density, movement, tension, darkness, space
- explore sound variations
- anchor current sound
- switch visual scene
- disable visuals
- start/stop recording

Implemented macro words:

- `ambient`
- `dark`
- `bright`
- `subby`
- `heavy`
- `wide`
- `space`
- `tense`
- `uneasy`
- `calm`
- `low_arousal`
- `no_visuals`
- `visuals`
- `black_white`
- `slow_orbits`
- `play`
- `stop`

`POST /llm` currently recognizes phrases around:

- ambient / drone / Eno
- dark / bright
- wide / space
- sub / heavy / bass
- tense / uneasy / uncanny
- calm / low arousal / slow
- visuals / no visuals
- play / start / run / record
- stop

Do not expose:

- arbitrary filesystem writes
- shell commands
- plugin scanning
- network calls from the LLM response
- direct audio callback access
- arbitrary Rust/plugin code

## Local Process Option

For OpenCode or a custom local planner, SoundWorld can call a process:

```text
SoundWorld sends project summary JSON to stdin.
Planner returns AiPlan JSON to stdout.
SoundWorld validates and applies the commands.
```

This avoids hard-coding one provider and keeps API keys outside the audio app.

## Next Version

- chat panel in the GUI
- provider config for disabled/local/OpenCode/OpenRouter/OpenAI
- external OpenAI/OpenRouter/OpenCode process adapters
- review-before-apply UI
- musical scheduling

That is more work, but it builds on the same command/event layer.
