# LLM Integration Plan

SoundWorld can become AI-native, but it should not put an LLM in the realtime audio engine.

The correct architecture:

```text
chatbar
  -> LLM provider or local parser
  -> proposed AiPlan JSON
  -> validation
  -> typed Command list
  -> Event log
  -> scheduled state changes
```

## Why This Works

The app now has:

- `Project`
- `Command`
- `Event`
- `Transport`
- `AiRequest`
- `AiPlan`
- `AiProvider`

That means a chatbar can eventually ask:

```text
start ambient track with these sounds, no visuals
```

and the planner can return:

```json
{
  "summary": "Start ambient generation using anchored sounds and disable visuals.",
  "commands": [
    { "Transport": "Play" },
    { "Visual": { "SetScene": { "name": "disabled" } } }
  ],
  "confidence": 0.82
}
```

The exact JSON shape will use Serde's Rust enum format, but the point is the same: the LLM returns commands, not arbitrary code.

## Provider Choices

### Disabled

Default. No network. No API key. Local command parser only.

### OpenCode

Use opencode as an external planning process.

Possible flow:

```text
SoundWorld writes request JSON to stdin
opencode/provider reads state summary
opencode/provider writes AiPlan JSON to stdout
SoundWorld validates
```

This is good for local experimentation because SoundWorld does not need to know much about the provider.

### OpenRouter / OpenAI

HTTP API adapters behind optional features.

Important:

- API keys should come from environment variables or config outside project files.
- No secrets in Git.
- Network errors must degrade gracefully.

### Local Process

Any executable that accepts a request and returns a plan.

This allows future local small models without changing SoundWorld.

## Safe Command Allowlist

Initial allowed tools:

```text
InspectProject
SetParameter
RampParameter
ExploreSound
AnchorSound
SetDensity
SetTension
SetMovement
ChangeVisualScene
ScheduleEvent
```

Do not allow:

```text
filesystem writes
shell commands
network calls from the plan
audio callback calls
plugin scanning
arbitrary code
```

## First Useful Chatbar Features

1. Local parser handles common phrases.
2. AI provider is optional.
3. Chatbar shows proposed commands before applying.
4. User can approve/reject.
5. Accepted commands go into `Project.history`.

Example prompts:

```text
start ambient no visuals
make it darker over 8 bars
less dense but more movement
explore this bass but keep it heavy
anchor this and branch again
switch to a sparse drone section
```

## Implementation Order

1. Build `AiPlan` validation.
2. Add `Chat` tab or expandable chatbar.
3. Convert current command parser into a `LocalPlanner`.
4. Add provider config:

```json
{
  "provider": "Disabled",
  "review_before_apply": true
}
```

5. Add `LocalProcess` provider.
6. Add opencode adapter.
7. Add HTTP adapters only after local planning works.
