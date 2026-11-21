# Roadmap

Each phase ends with a written report in `docs/reports/`, a "Questions and ideas for next phase" section,
and a stop for your approval. Exit criteria are measurable. Numbers are first targets and may be revised
with your approval.

| Phase | Name | Status |
|---|---|---|
| 0 | Design document | Approved |
| 1 | Headless Minds Engine | **In review** (4/5 exit criteria; see reports/phase-1-report.md) |
| 2 | Full evolutionary arc (headless) | Not started |
| 3 | Civilization (headless) | Not started |
| 4 | Renderer and assets | Not started |
| 5 | Broadcast layer | Not started |
| 6 | Tank News Network | Not started |
| 7 | Shrink team and First Contact | Not started |

---

## Phase 0: Design document
**Deliverables:** `DESIGN.md`, `DECISIONS.md`, `ROADMAP.md`, `OPEN_QUESTIONS.md`, Phase 0 report.
**Exit:** your approval of the design, or a list of changes.

## Phase 1: Headless Minds Engine (Rust)
**Scope**
- Cargo workspace `sim/`: world core (simplified tank: terrain heightfield, light/temperature/food fields,
  Midwest climate driver with a bundled dataset), deterministic RNG streams, snapshots, event log.
- Mind module (DESIGN §7) at full sapient capacity, *with capacity scaling already wired* (D-005).
- Cooperation, groups (detected + believed), roles (detected + believed), naming-game lexicon +
  phonology + sound change (§9; audio not needed yet).
- Life records, notability, thought log.
- Discovery Registry + a **small** discovery subgraph (~25 nodes: tools, storage, shelter, burial, simple
  norms) to test need-driven experimentation and knowledge loss.
- Genome type defined in full; sapients seeded from a hand-tuned distribution.
- CLI: `tank run --seed N --config X --cycles C`, `tank batch --seeds 1..200 --jobs 4`, `tank inspect`,
  `tank thoughts --agent ID`, `tank replay --to TICK`.
- Determinism test (same seed → same hash; replay from snapshot matches).
- `tools/bench/`: throughput and memory benchmark you can run on the Mac mini.

**Scenario tests:** famine, predator surge, a stranger arriving, a leader's death, resource windfall, plague
(simple pathogen model), plus **harsh Midwest year** (a drought or extreme-winter block).

**Metrics in every batch report:** survival curves; cooperation rate; group formations/splits; distinct roles
detected; belief diversity (entropy of belief sets); discoveries per generation; knowledge lost; lexicon
size/coherence/grammar tier and inter-group intelligibility; **behavior entropy** across individuals;
**change-per-cycle** (an anti-equilibrium metric); decisions/sec.

**Exit criteria**
- 200-mind, 20-cycle tank runs in < 5 min on one core (dev container) and is deterministic.
- Across 100 seeds: groups form *and* split in ≥ 60%; ≥ 4 distinct roles emerge in ≥ 50%; at least one
  culture-specific belief that is *wrong* appears in ≥ 50%; ≥ 2 mutually unintelligible lexicons in ≥ 30%
  of seeds with geographic separation; knowledge is lost at least once in ≥ 30%.
- I read sampled thought logs and judge them believable/varied (written assessment with excerpts).
- Each scenario test produces a qualitatively different response from baseline.

## Phase 2: Full evolutionary arc
**Scope:** chemistry fields, strains, super-individual organisms, genome evolution, speciation detection,
capacity-scaled minds for animals, fossils + strata, storyteller, era detection, end conditions.
**Exit:** across ≥ 200 seeds, era-reach distribution within ±50% of DESIGN §21 targets; speciation and mass
extinction events observed; sapience arises in some seeds and not others; strain/agent counts stay bounded;
deep-time fast-forward reaches minds within the 6–48 h wall-time window on the Mac mini (projected from
benchmarks).

## Phase 3: Civilization
**Scope:** full discovery graph (150–250 nodes), knowledge spread/loss/rediscovery (incl. via ruins),
settlements, specialization, trade, leadership, law, belief systems, conflict, collapse dynamics, style
genome, watcher awareness.
**Exit:** first **ranked Discovery Registry report** across ≥ 200 seeds; civilizations follow different paths
(path diversity metric); at least one documented collapse and one dark age → rediscovery; style genomes
measurably diverge and blend on contact; watcher beliefs appear in < 5% of civilized tanks.

## Phase 4: Renderer and assets
**Scope:** three.js aquarium, creature/structure procedural generators, style application, Asset Library,
hot-swap, Camera Director (aquarium + event modes, strata cutaway), lookahead playback, pre-launch asset batch.
**Exit:** 60 fps at 1080p target / ≥ 30 fps floor on the Mac mini with typical load; every Registry
descriptor renders procedurally; hot-swap demo; 1-hour unattended camera recording reviewed for jitter/legibility.

## Phase 5: Broadcast layer
**Scope:** overlays, Chronicle (templates), filler segments, Hall of Fallen Tanks, live outside-weather chip,
inhabitant voice audio, OBS scenes, Twitch + YouTube simultaneous streaming, supervisor/watchdog, health page.
**Exit:** **72-hour soak** on the Mac mini with memory slope, CPU/GPU and FPS report; automatic recovery from
killed sim/hub/renderer/OBS verified.

## Phase 6: Tank News Network
**Scope:** llama.cpp vs MLX benchmark, script pipeline with fact IDs + grounding, news team, schedule
(America/Chicago), breaking news, Kokoro TTS, replay-buffer packages, weather segment with forecaster accuracy.
**Exit:** 100% of aired sentences pass grounding (fallbacks counted); every scheduled broadcast airs over a
7-day soak; no frame drops attributable to LLM (FPS heartbeat log).

## Phase 7: Shrink team and First Contact
**Scope:** observation pod, glimpses, language gate, deterministic contact roll, grounded interviews with
native-word passthrough, bounded belief rule table.
**Exit:** contact occurs only when gated; interview grounding audit passes; belief-effect outcomes match
the rule table across forced-scenario tests.
