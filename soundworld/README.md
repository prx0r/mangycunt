# SoundWorld

Lightweight AI-native audiovisual instrument prototype for old x86_64 Linux laptops.

SoundWorld is the first prototype of Mangy: a standalone instrument for making a bass sound, exploring variations, generating a simple ambient world, nudging it live, and recording the result.

This is not a DAW clone and it does not host Surge XT/Cardinal plugins yet. Use Ardour/REAPER for plugin hosting now; use SoundWorld as the custom instrument/runtime that will later control or integrate external synths.

The first vertical slice includes:

- native egui/CPAL app
- bass-oriented synth controls
- deterministic patch save/load
- bounded patch mutation and 2D SoundWorld candidates
- simple continuous ambient generator
- typed local command nudges
- procedural music-driven visual panel
- WAV session recording
- a built-in Guide tab
- a safe AI-planner architecture boundary

Run:

```bash
scripts/bootstrap-linux.sh
scripts/run.sh
```

Project state is JSON-first. User data defaults to:

```text
~/.local/share/soundworld/
~/.config/soundworld/hardware.json
```

Read next:

- [AGENT_HANDOFF.md](AGENT_HANDOFF.md)
- [USER_GUIDE.md](USER_GUIDE.md)
- [LOCAL_TEST_STATE.md](LOCAL_TEST_STATE.md)
- [TESTING_NOTES.md](TESTING_NOTES.md)
- [API_CONTROL.md](API_CONTROL.md)
- [AI_DAW_RESEARCH.md](AI_DAW_RESEARCH.md)
- [HARMONY_AGENT_RESEARCH.md](HARMONY_AGENT_RESEARCH.md)
- [CORPUS_WORLDS.md](CORPUS_WORLDS.md)
- [AI_PLANNER.md](AI_PLANNER.md)
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [RESEARCH_NOTES.md](RESEARCH_NOTES.md)
