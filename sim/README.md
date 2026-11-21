# tank-sim: the headless engine

Deterministic Rust simulation of THE TANK. Phase 1 implements the **Minds Engine** on a simplified
2-D tank (see `docs/DESIGN.md` §7–10 and `docs/reports/phase-1-report.md`).

## Build

```bash
cargo build --release          # binary: target/release/tank
cargo test --release           # determinism, snapshot/restore, language tests
```

## Commands

| Command | What it does |
|---|---|
| `tank run --seed 7 --cycles 40` | One tank. Writes `runs/seed-7/`: `summary.json`, `cycles.csv`, `events.jsonl`, `chronicle.txt`, `thoughts.txt`, `people.json`, `registry.json`, `languages.json` |
| `tank run ... --viewer [--viewer-cycles 0..12] [--viewer-stride 128]` | Also writes `viewer.json` for the debug viewer (`tools/viewer/index.html`) |
| `tank run ... --scenario famine@6` | Inject a test scenario at cycle 6 (`famine`, `predator_surge`, `stranger`, `leader_death`, `windfall`, `plague`, `harsh_winter`, `drought_year`). Comma-separate for several. |
| `tank run ... --thoughts all\|tracked\|none` | Thought-log volume (default: 12 tracked people + anyone notable) |
| `tank batch --seeds 1..24 --cycles 60 --jobs 4` | Many tanks in parallel → `report.md` with Phase 1 exit criteria and metric distributions |
| `tank scenarios --seeds 1..6 --cycles 30 --at 8` | Baseline + every scenario on the same seeds → `scenarios.md` comparison |
| `tank determinism --seed 3 --cycles 8` | Same seed twice + snapshot/restore; exits non-zero on mismatch |
| `tank bench --pops 100,200,400,800` | Decisions/sec, µs/decision, sim-days/sec and peak RSS by population |
| `tank inspect --run runs/seed-7 --agent 12` | One person's life story, beliefs, rules, relationships and recent thoughts |
| `--config config/presets/harsh.toml` | Any command: override parameters (see `src/config.rs`) |

## Layout

| File | Contents |
|---|---|
| `sim.rs` | World + people + deterministic main loop, daily routines, predators, disease, births, deaths, knowledge census, snapshots |
| `decide.rs` | Utility decisions, anticipatory intents, requests/responses, joint hunts, naming game, gossip, teaching, experimentation, building, theft, punishment |
| `mind.rs` | Drives, emotions, episodic memory, beliefs, contingency learning (prediction), theory of mind |
| `language.rs` | Phonology, word coining, sound laws, lexicons |
| `discovery.rs` | The 26-node aquatic discovery subgraph and the Discovery Registry |
| `social.rs` | The observer: groups, roles, languages, norms, belief statistics (read-only) |
| `climate.rs` | Chicago climate driver (one tank cycle = one real year) |
| `world.rs` | Tank map, habitats, food regrowth, gulpers, drifter herds, camps |
| `scenario.rs` | Test-only scenario injections |
| `export.rs`, `report.rs`, `metrics.rs` | Outputs, reports and metrics |
