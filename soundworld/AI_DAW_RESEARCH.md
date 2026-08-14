# AI DAW / Synth Control Research

Date: 2026-08-15

The useful pattern from current AI music tools is consistent: do not put a model in the realtime audio path. Let AI inspect state, propose typed actions, and leave normal editable musical material behind.

## Comparable Systems

- AbleMind: Ableton co-pilot using natural language for transport, MIDI generation, device parameter control, sound design, and effects chains. Source: https://ablemind.live/
- Yuma: Ableton assistant focused on editable MIDI and clip automation such as filter sweeps, volume rides, and panning. Source: https://www.yuma.studio/blog/yuma-writes-automation-in-ableton
- Ableton MCP / Live MCP: exposes Ableton session objects to AI assistants through MCP, often using a local bridge and typed tool calls. Sources: https://github.com/uisato/ableton-mcp-extended and https://live-mcp.mixofreality.studio/
- ChatM4L: Max for Live chat device that creates MIDI, controls parameters, and works with multiple model providers. Source: https://chatm4l.com/
- nob: intelligent synth plugin where natural language creates and refines a playable synth patch. Source: https://www.nob.audio/
- Deep Noise: AI synth plugin workflow where generated sounds remain playable in the DAW. Source: https://docs.deepnoise.ai/product-guides/quick-start-tips/working-in-daw
- Suno Studio 2.0: adds more DAW-like MIDI, automation, synth, and chat features, but still does not host normal third-party plugins. Source: https://www.theverge.com/ai-artificial-intelligence/979345/suno-studio-2-0-midi-chatbot-custom-effects
- Magenta RealTime 2: live AI instrument direction: text plus MIDI/audio control, designed for low-latency interaction. Source: https://magenta.withgoogle.com/magenta-realtime-2

## Mechanisms Worth Borrowing

1. Typed tool surface

Use typed commands instead of arbitrary natural language execution. SoundWorld now has this with `POST /commands`.

2. Editable output

AI should produce normal state changes: notes, patch parameters, automation curves, scene changes, anchors, and events. Do not only generate rendered audio.

3. Parameter discovery

Future DAW/plugin integration should expose a normalized list of controllable parameters. For Surge/Cardinal this likely means controlling the DAW/plugin host by MIDI CC, OSC, CLAP/VST host automation, or a DAW scripting bridge rather than directly embedding them first.

4. Automation lanes

Add commands for timed ramps and envelope points. This is one of the most useful features in Yuma/Ableton-style systems because it gives motion, not just notes.

5. Review before apply

For destructive or broad actions, the LLM should show the proposed plan before applying. For safe live actions like play/stop/density/visuals, direct apply is acceptable.

6. Local-first bridge

Bind to localhost and keep providers optional. This matches the MCP/local bridge pattern and avoids putting API keys or network calls in the audio thread.

7. Prompt-to-patch macros

Borrow from nob: map words like darker, sharper, heavier, wider, less fuzzy, more metallic into grouped parameter moves. SoundWorld already has the start of this through `Music::Nudge`; the next step is named macro definitions.

## Integrated In This Pass

- Localhost API server at `127.0.0.1:3769`.
- `GET /health`.
- `POST /commands`.
- AI-origin typed commands routed into the same `Project.accept_command(...)` path as GUI actions.
- Live GUI smoke test driven by `curl`.
- App-state unit test verifying transport, density, visual disable, exploration, and AI-origin event logging.

## Next Sick Mechanisms

- `AutomationCommand::AddPoint` and `AutomationCommand::Ramp` for filter sweeps and movement over bars.
- `/state` endpoint so an LLM can inspect current patch, transport, events, anchors, and candidates before planning.
- `/audition` endpoint that triggers a note and optionally writes a short WAV preview for automated validation.
- Macro map: `heavy`, `dark`, `sharp`, `wide`, `clean`, `dirty`, `ambient`, `acid`, `subby`.
- Optional MCP server wrapping the same local API so Claude/Codex/OpenCode-style agents can control SoundWorld through standard tools.
