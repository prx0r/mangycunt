# Mangy / SoundWorld

This repo is being moved from a lightweight standalone synth prototype toward a real DAW stack.

Current split:

- `soundworld/`: Rust `egui`/`cpal` prototype for AI-native sound/vector exploration.
- `docs/OPENDAW_INTEGRATION.md`: the proper DAW direction using OpenDaw, Qt6, JUCE, and Tracktion Engine.
- `scripts/setup-opendaw-stack.sh`: reproducible setup/build attempt for OpenDaw on Linux.

The practical goal is:

```text
OpenDaw hosts Surge XT/Cardinal
SoundWorld/Mangy provides AI control, harmony/vector generation, and future visuals
agent API can create a short project and render WAV proof
```

Start here:

```bash
./scripts/setup-opendaw-stack.sh
```

Then read:

- `docs/OPENDAW_INTEGRATION.md`
- `docs/OTHER_REPOS_ROADMAP.md`
- `docs/DAW_VALIDATION.md`
- `soundworld/AGENT_HANDOFF.md`
