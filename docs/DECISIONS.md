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

---

## Phase 1 decisions (2026-11-21)

### D-024 · Phase 1 world is a 2-D top-down grid
**Status:** Proposed
**Decision:** 64×32 cells (8 units each) grouped into 32 named "places"; depth, habitat, materials, regrowing algae/shellfish, vents, a rock ridge with two passes. The 3-D field model (DESIGN §3.2) arrives in Phase 2.
**Why:** Phase 1 is about minds; a 2-D map makes geography matter (travel, separation, local depletion) at a fraction of the cost.
**Rejected:** Building the full 3-D fields first (delays the minds work; nothing in Phase 1 needs vertical structure).

### D-025 · One thread per tank; parallelism across tanks
**Status:** Proposed
**Decision:** A tank runs single-threaded; `tank batch` runs tanks in parallel.
**Why:** Simplest way to guarantee determinism; batch testing is where throughput matters now. Revisit in Phase 2 if one tank needs more than one core (region-parallel design in DESIGN §18.4 still applies).

### D-026 · Climate: shuffled multi-year blocks of real Chicago years
**Status:** Accepted (implemented as designed)
**Decision:** NOAA GHCN-Daily Midway (+ O'Hare fill), 1929–2025, bundled as CSV with a recorded checksum. 365 real days → 48 tank days; extremes preserved.

### D-027 · Language stability: hearers supply missing words; entrenched words resist replacement
**Status:** Proposed
**Decision:** A speaker with no word for a concept adopts the hearer's (coins only if neither has one). New synonyms are coined at 0.06% of utterances, scaled down for entrenched words. Children mishear 2% of new words. Sound laws spread word by word.
**Why:** Without these, vocabularies churned so fast that a single camp had six words for "food". Real basic vocabulary is stable within a community and drifts between separated ones.

### D-028 · Intelligibility = recognition; languages = average-linkage clusters
**Status:** Proposed
**Decision:** Community B understands A to the extent that B's members recognise A's usual words (any form they know for the concept, within one sound), counting only concepts B has a word for. Communities join a language when their average intelligibility with it is ≥ 0.5.
**Why:** Earlier measures (modal-word comparison, complete linkage) counted vocabulary gaps and within-group noise as separate languages (one tank reported 32 languages).

### D-029 · Use it or lose it
**Status:** Proposed
**Decision:** Every discovery a mind holds fades daily unless practised (tools faster than practices/beliefs); below a threshold it is forgotten. Practising a tool also refreshes its prerequisites.
**Why:** Without forgetting, knowledge once learned spread to everyone forever and was never lost. Now 96–100% of tanks lose something and ~90% rediscover something: dark ages and recoveries.

### D-030 · Hearsay never outruns the teller; beliefs fade; experience disconfirms
**Status:** Proposed
**Decision:** A belief received by gossip is capped at 85% of the teller's confidence; unreinforced beliefs fade (hearsay and superstitions faster); successfully using a "cursed" material weakens the curse; kindness weakens a thief rumour.
**Why:** Superstitions and accusations saturated at 100% across whole populations, which also shut down invention (no one would touch "cursed" materials).

### D-031 · Gulpers prefer drifters; take the isolated and the benighted
**Status:** Proposed
**Decision:** Hungry gulpers usually hunt drifter herds; they target people who are alone (or anyone at night), are deterred by crowds and walls, breed slowly, and drift back in from the wider tank when scarce. A lone person rarely kills one without a spear.
**Why:** Tuning swung between "people mob every gulper to death" and "gulpers eat everyone in one cycle". Current: gulpers cause ~30% of deaths.

### D-032 · Infants are fed by parents first, then by kin and generous camp-mates
**Status:** Proposed
**Why:** Initially ~75% of births died of starvation because parents gave their stash away. Parents with dependents now hold back from non-kin.

### D-033 · A habit term in utility
**Status:** Proposed
**Decision:** Activities a person practises often get a small utility bonus (excluding eating, sleeping, storing).
**Why:** Lets specialists emerge from practice rather than assignment.

### D-034 · Scenarios are test inputs only
**Status:** Proposed
**Decision:** `famine`, `predator_surge`, `stranger`, `leader_death`, `windfall`, `plague`, `harsh_winter`, `drought_year` exist only for headless experiments; broadcast tanks never run injections.

### D-035 · Roles: "lawgiver" is separate from "prophet"
**Status:** Proposed
**Decision:** A person whose norm (e.g. "stealing is forbidden") spreads to ≥ 6 people is a lawgiver; a person whose belief about the unseen (curses, ancestors, the Lamp) spreads is a prophet.

### D-036 · Thought logs: full for a tracked sample + notables, ring buffers for everyone
**Status:** Proposed
**Decision:** Every mind keeps its last 64 meaningful thoughts; 12 tracked individuals and anyone notable are logged in full to `thoughts.txt`.

### D-037 · Founding population is 80 in four bands of 20
**Status:** Proposed (OQ-14, your default)
**Decision:** Start with 80 founders (20 per band, close to real foraging-band sizes), food stocks near equilibrium (algae 35%, shellfish 45% of capacity), and conception only when the household has food stored.
**Why:** 120 founders exceeded what the starting map can feed, so every tank crashed (median 126 → 49) in its first cycles. That was an artefact of the starting conditions, not tank dynamics. Now: 83 → ~96 → settles near 75, 0% extinction by cycle 30.

### D-038 · Algae gardens require lamp-watching and a camp of 6
**Status:** Proposed (OQ-15, your default)
**Why:** Cultivation was reachable from nothing and caused very early booms. Now it needs a sense of the seasons first.

### D-039 · Leadership has real effects, through prestige
**Status:** Proposed (OQ-16, your default)
**Decision:** Each mind tracks its own *prestige* (others accepting its proposals, following it, learning from it, its hunts and gulper kills; it fades). Minds with high prestige punish thieves who didn't rob them, redistribute food and organise hunts more. Followers favour requests from someone they believe leads and copy what respected people do (prestige-biased imitation, a new learning channel). Followers of a dead leader become restless.
**Result:** losing the most respected person now costs a tank about 11 people, more starvation, fewer inventions and more theft. Before, it had no effect.

### D-040 · Norms: witnessing is first-hand; victims learn; victims react on the spot
**Status:** Proposed
**Decision:** Seeing a thief punished is filed as one's own experience (not hearsay); being robbed teaches that robbing is wrong; a victim who catches a thief is interrupted and decides immediately whether to punish. Norm beliefs fade slowly.
**Result:** an anti-theft norm takes hold in 56% of tanks (was 4–12%), with about 11 punishments per tank and no feud spiral.

### D-041 · Capacity profiles (implements D-005)
**Status:** Proposed
**Decision:** At birth each mind gets limits and features from its `capacity` gene. Memory, belief, acquaintance and vocabulary limits scale with (capacity − 0.15)/0.70 squared, so config values are the limits of a typical sapient (0.85). Learned language and gossip switch on at 0.35; planning, teaching and experimenting at 0.55. Low-capacity minds need stronger evidence before trusting a learned rule. Kin are always remembered, so the acquaintance limit applies to non-kin.
**Why:** Phase 2 must run animals and sapients through the same mind code at different capacities. Founders (0.78–0.95) now differ slightly in how much they can hold, which adds individual variation.

### D-042 · Group identity: "we are the people of X"
**Status:** Proposed
**Decision:** An adult mind with language holds an identity with its home camp. It grows with time among camp-mates it likes, only if the mind has a word for the camp (a shared name), and is strengthened by successful joint hunts. When someone moves, the old allegiance fades before the new one forms. Effects: favours to and from "us", wariness of "them", the anti-theft norm binding less toward outsiders, outsiders punished more readily, and loyalty making people slower to leave. The observer reports believed communities (most adults strongly identify) separately from detected groups, plus how well the two agree, and logs when a camp first sees itself as a people.
**Result:** founding bands become named peoples within 1–2 cycles; breakaway camps form their own identity later; detected groups and believed identities agree about 70% of the time. Punishment rises (inter-group friction), and so does extinction, which D3/D4 address.

### D-043 · Escaping boom-bust: restraint at harvest and in demand
**Status:** Proposed (OQ-17)
**Decision:**
- *Bed-resting* (discovery 26, a practice invented under hunger): knowers skip worn beds and leave the rest half-grown.
- *Foresight in births:* a mind that can plan and expects hunger (from its own learned rules, e.g. "crowded camp → hunger") is less likely to conceive. "Crowded" now fires at realistic camp sizes (over 25, or over 10 when home food looks poor).
- *Gulpers follow prey:* they drift in from the wider tank in proportion to the drifters present; emptied tanks are re-colonised by drifters a little more often.
- *No vendettas:* a grudge alone licenses only immediate retaliation; punishment settles the debt (after witnesses have judged it).
**Why:** Bed-resting alone spread everywhere by cycle 3 and changed nothing, because births kept growing. Gulper immigration ignored prey and finished off post-crash survivors. A feud spiral (up to 687 punishments per tank) had been suppressing growth and inflating the escape rate.
**Result (24 seeds × 60 cycles, `phase-1-batch-d3d4.md`):** 29% of booming tanks escape, 58% crash (half of those regrow past the old peak), 0% extinct. Thefts fell from about 50 to about 20 per tank; anti-theft norms take hold in 21%.

---

## Phase 2 decisions (in progress)

### D-044 · Evolution mode and 2.5-D water
**Status:** Accepted (Phase 2 Q1–Q3)
**Decision:** `config/presets/evolution.toml` starts a lifeless tank; the default "start with people" mode is unchanged. Each map cell has 3 water layers (surface, mid, bottom) with light, temperature, oxygen, nutrients and detritus. Hand-written gulpers and drifters exist only in "start with people" mode.

### D-045 · Microbial evolution: strains, arms race, bounded table
**Status:** Proposed
**Decision:** Up to 4 strains per cell-layer; life starts where prebiotic soup is rich (vents or sunlit, nutrient-rich shallows). Size needs multicellularity, and a grazer can only eat what it is big enough to eat; size and multicellularity cost steeply. Near-identical strains within a species are merged so the table never blocks new mutants. A mass extinction is a collapse in living species (below half the recent peak), not turnover.
**Why:** Without the arms race, multicellularity never evolved; without merging, evolution froze once the table filled.

### D-046 · Animals as herds of evolving creatures
**Status:** Proposed
**Decision:** Dense multicellular grazers can give rise to animals: agents standing for herds, each with a full genome (size, diet, speed, armour, senses, intelligence, sociality, preferred depth). Herds feed across an area that grows with herd size, follow food gradients (even without intelligence), avoid crowded spots, breed in proportion to local food per head, split when large, and leave grazing refuges (beds and mats are never stripped to zero). Kin are recognised by genetic similarity; anything more distant can be prey. Intelligence costs energy and lengthens childhood, and buys wider search, memory, better escapes and better hunting.
**Why:** Each rule fixed an observed failure: herds starving on one depleted cell, a food-accounting error (herd intake didn't scale with herd size), fractional herds leaking animals, hunting impossible within one species, and grazers wiping out their own food base.
**Status of results:** animals persist in 3/4 test tanks to cycle 800; intelligence reaches 0.35 occasionally; no dedicated hunters yet.
