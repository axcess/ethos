# Decisions Log

Every significant decision, with its reasoning and the alternatives I rejected. Newest at the bottom.
Status: **Accepted** (approved by you), **Proposed** (my call, awaiting review), or **Superseded**.

---

### D-001 · Prime Directive is absolute
**Status:** Accepted (Q-defaults, 2026-11-21)
**Decision:** The shrink team never interferes. The only effects on the world are (a) rare glimpses feeding watcher awareness and (b) bounded First Contact belief rules.
**Why:** Pillar 1 (zero player). An "accident" mechanic would need a non-simulation source of decisions.
**Rejected:** Rare rogue-reporter accidents. They could be revisited later as a deterministic in-simulation probability, but not now.

### D-002 · Pacing: 1–3 weeks per civilizing tank
**Status:** Accepted (Q1)
**Decision:** Deep time in fast-forward for the first 6–24 h; minds by day 1–2; civilization over ~10+ days. Time units: 1,024-tick day, 12-day season, 48-day cycle.
**Why:** Keeps most stream hours on living, legible creatures while preserving the full arc. A short year makes seasons and weather watchable.
**Rejected:** Real-time-ish evolution (months of ooze); 365-day years (seasons too slow to read on stream).

### D-003 · Fully aquatic tank with a hostile, reachable surface
**Status:** Accepted (Q2)
**Decision:** No fire. Water-native technology (vent smithing, glowlamps, reef cultivation, current mills, air bladders). Breaking the surface is a rare milestone.
**Why:** More original, keeps glass and light motifs central, and makes the discovery graph distinctive.
**Rejected:** Paludarium with a shoreline (more familiar tech, but splits the world in two and dilutes the aquarium identity).

### D-004 · Tiered levels of simulation with promotion by evolved traits
**Status:** Accepted (challenge A)
**Decision:** Chemistry fields → strain populations → super-individual organisms → partial minds → full minds. Promotion is driven by gene thresholds, never timers or eras.
**Why:** The only way to span microbes to cities on one machine while keeping outcomes emergent.
**Rejected:** One uniform agent model (too slow or too shallow); scripted era transitions (violates pillar 2).

### D-005 · One Mind module, capacity-scaled
**Status:** Proposed
**Decision:** A single Mind implementation whose features scale continuously with `neural_capacity`. Social animals and sapients run the same code.
**Why:** Phase 1 minds automatically become Phase 2 animal minds. Intelligence evolves gradually instead of switching on.
**Rejected:** Separate animal-AI and sapient-AI codebases (duplication, a visible "jump" at promotion).

### D-006 · Cross-platform bit-exact determinism
**Status:** Proposed
**Decision:** Same seed + config + engine version + climate dataset → identical state on macOS arm64 and Linux. Pure-Rust `libm`, no unordered iteration, region-ordered parallel merges, CI dual-arch hash test.
**Why:** I develop on Linux and you run on macOS. Replays, regression tests and crash recovery all depend on this.
**Rejected:** Same-platform-only determinism (bugs found on the Mac couldn't be reproduced in dev); fixed-point math everywhere (slower to write, little extra safety).

### D-007 · Body-plan gate for building and legibility
**Status:** Accepted (Q3)
**Decision:** Building requires an evolved `manipulator` trait. Readable sapients need `expressive_face`. Both are evolvable with independent fitness value.
**Why:** Legibility (pillar 4) without forcing a body template; handless smart species remain a valid outcome.
**Rejected:** Free-for-all body plans (illegible); fixed humanoid template (contradicts evolution).

### D-008 · Fixed-layout float genomes
**Status:** Proposed
**Decision:** ~96 named f32 loci in modules, Gaussian mutation, module duplication, uniform crossover, mating-signature isolation.
**Why:** Fast, compact, easy to diff, visualize and map to meshes.
**Rejected:** Graph or neural genomes (NEAT-style): richer but slow, opaque and hard to render.

### D-009 · Finished assets are parametric TS generators and/or glTF parts
**Status:** Accepted (Q7)
**Decision:** The Asset Library stores `generator.ts` and/or `.glb` + `meta.json` + thumbnail, in git.
**Why:** Claude Code writes code well; parametric assets accept style genomes natively.
**Rejected:** Requiring hand-made or AI-generated mesh art as the primary format (can still be added later as `.glb`).

### D-010 · Rust simulation + TypeScript everything else
**Status:** Accepted (Q4)
**Why:** Batch throughput, no GC pauses over weeks, multicore determinism; TS is needed for the browser anyway.
**Rejected:** All-TS on Bun/Node (≈3–10× slower batches, GC risk); C++; Zig.

### D-011 · End conditions fixed in config before the run
**Status:** Accepted (Q5)
**Decision:** Sterile, extinction, stagnation (sim-time or real-day limits), optional max lifetime (off by default).
**Why:** Prevents endless ooze while staying zero-player (rules set in advance, not mid-run).

### D-012 · Lookahead buffer: the simulation runs ahead of the broadcast
**Status:** Proposed
**Decision:** The simulation computes 60–180 s of wall time ahead; the hub plays back from the buffer.
**Why:** The camera can arrive before an event and time can slow *before* a climax. Presentation only, so determinism is intact.
**Rejected:** Reactive camera only (always arrives late, misses firsts).
**Cost:** Hub memory for the buffer (~100–300 MB est.) and a slight delay between sim and screen (irrelevant to viewers).

### D-013 · Harshness target distribution
**Status:** Accepted (Q6)
**Decision:** ~50% of tanks reach social, ~25% cultural, ~10% civilizational, ~2% of those reach First Contact.

### D-014 · Scale targets for minds
**Status:** Accepted (Q9)
**Decision:** 2–5k full-detail sapient minds, hard cap ~10k; distant/unwatched minds use a coarser heartbeat (same rules).

### D-015 · Monorepo layout
**Status:** Accepted (Q10)
**Decision:** `sim/` (Rust), `hub/`, `renderer/`, `content/`, `assets/`, `protocol/`, `config/`, `data/`, `tools/`, `docs/`. Working title "The Tank"; repo stays `ethos`.

### D-016 · Event-driven minds (~150 decisions/day)
**Status:** Proposed
**Decision:** Minds re-decide on action completion, interrupts, or a heartbeat, not every tick.
**Why:** 10–100× cheaper than per-tick thinking; matches how actions take sim-minutes. The renderer interpolates movement.
**Rejected:** Per-tick utility evaluation (wasteful), fixed low-rate ticking (sluggish reactions to threats).

### D-017 · Midwest weather: historical driver + live broadcast flavor
**Status:** Proposed (your mid-turn request; needs your confirmation, OQ-1)
**Decision:** Bundled NOAA GHCN-Daily Chicago records drive tank climate (one tank cycle = one real year, seed picks shuffled multi-year blocks). Live NWS conditions appear on screen only.
**Why:** Real-feeling weather with real extremes, while preserving zero-player, offline runtime and deterministic replay.
**Rejected:** Live weather as a simulation input. It would break replay/determinism (fixable only by logging inputs), adds a 24/7 network dependency, and is arguably an outside influence on the outcome (pillar 1).

### D-018 · Accepted pitches: spoken languages, fossils/strata, tank weather forecasting
**Status:** Accepted
**Decision:** See DESIGN §9.4, §11, §4.3.
**Scope notes:** Spoken language uses espeak-ng phoneme input (cheap). Fossils are bounded to ~20k buried items/tank. The forecaster runs in the hub, never in the simulation.

### D-019 · Plague is an organism
**Status:** Proposed
**Decision:** Pathogens are evolving Tier-1 strains with parasitic genomes, not status effects.
**Why:** Epidemics then emerge from density, immunity genes, hygiene beliefs and medicine discoveries, and pathogens can evolve.

### D-020 · Storyteller sees physics only
**Status:** Proposed
**Decision:** The storyteller module receives a read-only `PhysicalView` with no agent, group or culture data.
**Why:** Makes "never targets a group" true by construction, not by promise.

### D-021 · Runtime LLM: llama.cpp `llama-server`, ~8B Q4, JSON-schema-constrained
**Status:** Proposed (benchmark vs MLX-LM in Phase 6)
**Why:** Stable headless server, Metal acceleration, grammar/JSON-schema constraints for grounded scripts.

### D-022 · TTS: Kokoro-82M for news team, espeak-ng phonemes for inhabitants
**Status:** Proposed
**Why:** Kokoro is small, permissively licensed and has distinct voices; espeak-ng can speak arbitrary phoneme strings for invented languages. macOS `say` is the fallback.

### D-023 · Replay buffer stores state, not video
**Status:** Proposed
**Decision:** News packages re-render clips from stored state diffs + camera directives.
**Why:** Orders of magnitude smaller than video; any angle can be re-shot.
