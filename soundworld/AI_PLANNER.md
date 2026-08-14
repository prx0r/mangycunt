# AI Planner Boundary

Mangy can use an LLM, but the LLM must not run inside the realtime audio path.

The architecture should be:

```text
human text / voice
      ↓
LLM provider
      ↓
proposed typed commands
      ↓
schema validation
      ↓
project planner / scheduler
      ↓
Command -> Event -> Project
      ↓
audio / music / visual engines
```

The LLM is a planner, not the instrument.

## Provider Options

The provider should be abstract:

- `OpenCode`: call a local/opencode-controlled process if available.
- `OpenRouter`: generic HTTP API adapter.
- `OpenAI`: direct API adapter.
- `LocalProcess`: run a configured local command.
- `Disabled`: default, no network and no model required.

No provider should be mandatory for launching Mangy.

## Good First Use

Convert natural language into existing commands:

```text
"make it darker over four bars"
```

becomes:

```json
{
  "commands": [
    {
      "Music": {
        "Nudge": {
          "target": "darkness",
          "delta": 0.15,
          "beats": 16.0
        }
      }
    }
  ]
}
```

Then the normal command/event system handles it.

## What opencode could do

If opencode is available locally, Mangy can treat it as an external planner process:

```text
Mangy writes request JSON
      ↓
opencode/provider reads project summary
      ↓
provider writes plan JSON
      ↓
Mangy validates and schedules commands
```

This keeps secrets/API keys outside the audio engine and lets the user swap providers.

## Safety Rules

- The audio callback never calls an LLM.
- The LLM never writes project files directly.
- The LLM only proposes commands from an allowlist.
- Commands are validated before acceptance.
- Commands are scheduled on musical boundaries when relevant.
- Unknown or invalid plans are rejected silently or shown as errors.

## Research Next

- JSON schema validation for `AiPlan`.
- Provider subprocess protocol.
- OpenRouter/OpenAI HTTP adapter behind an optional Cargo feature.
- UI panel for reviewing proposed commands before applying them.
- Preference model from `Anchor`, `UseInTrack`, and `Reuse` events.
