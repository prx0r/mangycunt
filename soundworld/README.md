# SoundWorld

Lightweight AI-native audiovisual instrument prototype for old x86_64 Linux laptops.

This is not a DAW clone. The first vertical slice includes:

- native egui/CPAL app
- bass-oriented synth controls
- deterministic patch save/load
- bounded patch mutation and 2D SoundWorld candidates
- simple continuous ambient generator
- typed local command nudges
- procedural music-driven visual panel
- WAV session recording

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
