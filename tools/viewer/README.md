# Debug viewer (Phase 1)

A 2-D top-down view of a headless run, for reviewing behaviour. Not the broadcast renderer.

```bash
cargo build --release
./target/release/tank run --seed 7 --cycles 12 --viewer
python3 -m http.server 8000      # from the repo root
# open http://localhost:8000/tools/viewer/index.html?file=../../runs/seed-7/viewer.json
```

Or open `tools/viewer/index.html` directly and choose the `viewer.json` file.

- **Map:** habitats (shallows, sand, reef, kelp, deep, vents, the rock ridge), food per place (green tint),
  night shading, ice at the surface, camps (gold rings, member count, ✎ = tally-mark records),
  drifter herds (tan), gulpers (red triangles), people (dots; children smaller; yellow ring = fever; white = stranger).
- **Colour by:** detected group, home camp, founding culture, current action, or most prominent role.
- **Thoughts:** recent thought-log lines appear as bubbles; click a person to pin them in the inspector.
- **Events:** click an event to jump there and select its first participant.
- Keys: space = play/pause, ←/→ = step.
