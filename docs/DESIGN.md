# THE TANK — Design Document

**Status:** Phase 0 draft for review · **Last updated:** 2026-11-21
**Companion docs:** [`DECISIONS.md`](DECISIONS.md) · [`ROADMAP.md`](ROADMAP.md) · [`OPEN_QUESTIONS.md`](OPEN_QUESTIONS.md) · [`reports/`](reports/)

> A sealed aquarium world that runs forever, unattended, on one Mac mini. Life arises, evolves, may become
> sapient, may build civilizations, and may die. Nobody holds the controller. We just watch.

This document describes *what* each system is and *how* it fits with the others. Numbers marked **[tune]**
are first guesses that the Phase 1–3 test runs will calibrate. Section numbers in the original project
prompt are cited as **(P§n)**.

---

## Contents

1. [Guiding rules](#1-guiding-rules)
2. [Timescale and pacing](#2-timescale-and-pacing)
3. [The tank: world model](#3-the-tank-world-model)
4. [Climate: the Midwest record and tank weather](#4-climate-the-midwest-record-and-tank-weather)
5. [Levels of simulation and how each era emerges](#5-levels-of-simulation-and-how-each-era-emerges)
6. [Genomes](#6-genomes)
7. [Minds](#7-minds)
8. [Cooperation, groups and roles](#8-cooperation-groups-and-roles)
9. [Language: signals, lexicons and spoken sound](#9-language-signals-lexicons-and-spoken-sound)
10. [Discovery, knowledge and culture](#10-discovery-knowledge-and-culture)
11. [Fossils, strata and ruins](#11-fossils-strata-and-ruins)
12. [The storyteller](#12-the-storyteller)
13. [Events, history and the Discovery Registry](#13-events-history-and-the-discovery-registry)
14. [Interestingness, time control and the camera](#14-interestingness-time-control-and-the-camera)
15. [Renderer and asset system](#15-renderer-and-asset-system)
16. [Broadcast, Chronicle and Tank News Network](#16-broadcast-chronicle-and-tank-news-network)
17. [Shrink team, Prime Directive and First Contact](#17-shrink-team-prime-directive-and-first-contact)
18. [Architecture and processes](#18-architecture-and-processes)
19. [Language and runtime recommendations](#19-language-and-runtime-recommendations)
20. [Memory, CPU and GPU budget](#20-memory-cpu-and-gpu-budget)
21. [Tank lifecycle and endings](#21-tank-lifecycle-and-endings)
22. [Risks and mitigations](#22-risks-and-mitigations)
23. [Glossary](#23-glossary)

---

## 1. Guiding rules

The seven pillars from the prompt (P§2) are design constraints. Here is how each one is enforced
mechanically:

| Pillar | Enforcement |
|---|---|
| Zero player | The simulation process accepts **no input** after start except "stop". The hub, renderer, LLM and TTS only read from it. The Camera Director and speed controller change *playback*, never simulation state (§14). |
| Emergence over authorship | No event in the simulation code references a specific group, era or story beat. The storyteller (§12) chooses only *what kind* of natural event happens and *where in the physical tank*. It never chooses based on who lives there. A lint rule and a code-review checklist ban scripted triggers such as `if era == X then spawn Y`. |
| Failure is real | No rescue code paths. Populations can reach zero. Asserts in test builds catch any "minimum population" logic. |
| Legibility | Every system emits labeled events with a viewer-facing summary. The renderer shows state changes (§15). |
| Runs forever | Seeded deterministic simulation, snapshots, append-only logs with caps, a supervisor, and a nightly renderer reload (§18). |
| LLM narrates, never decides | No LLM output path reaches the simulation. First Contact belief effects are deterministic rules driven by simulation state (§17). |
| Discoveries never wait for art | Every descriptor renders through a procedural generator. Finished art is an optional upgrade (§15). |

**Determinism contract.** Given `(seed, config, engine version, bundled climate dataset)`, the simulation produces
bit-identical state on any supported platform (macOS arm64 and Linux x86-64/arm64). This allows replay, crash
recovery, regression tests and "what happened at cycle 412" debugging. See D-006.

---

## 2. Timescale and pacing

Decided in Q1 (D-002): **a tank that reaches civilization lives 1–3 weeks of stream time.** Primordial and
biological eras mostly pass in fast-forward within the first 6–24 hours. Minds usually appear by day 1–2.

### 2.1 Simulation time units

| Unit | Definition | Notes |
|---|---|---|
| **tick** | Smallest simulation step | 1 tick ≈ 1 sim-minute |
| **day** | One light cycle ("the Lamp" rises and sets) | 1,024 ticks [tune] |
| **season** | 12 days | Four per year |
| **cycle** (year) | 48 days = 4 seasons | Chronicle dates are "the 412th cycle" |
| **generation** | Varies by species | Sapient lifespan is about 30–70 cycles [tune] |

A 48-day year is short on purpose. It keeps seasons and weather readable on a stream while lifespans stay
long enough to follow a person for a day of viewing.

### 2.2 Wall-clock pacing targets

| Mode | Approx. sim speed | Feels like |
|---|---|---|
| Near-real-time (big moments) | 1 day ≈ 3 min | You can watch one person's afternoon |
| Ambient (default mind era) | 1 day ≈ 20–40 s | A cycle ≈ 15–30 min; a life ≈ 10–30 h of stream |
| Fast (quiet social/biological eras) | 1 day ≈ 1–3 s | Seasons flicker past |
| Deep time (primordial/early biological) | 1 cycle ≈ 1–10 s | Evolution montage |

Implications: a civilization spanning 10 days of stream covers about 500–900 cycles, or about 15–30
sapient generations. That is enough for culture, language drift and the rise and fall of settlements.
It is not enough for realistic cumulative technology, so the discovery system (§10) uses faster-than-history
rates on purpose.

### 2.3 Lookahead (pre-roll)

The simulation runs **ahead** of what viewers see, by about 60–180 s of wall time [tune]. The broadcast plays
from a buffer. This lets the Camera Director arrive *before* the first spear is thrown and lets the
speed controller slow down *before* a famous person dies (§14). This only affects presentation, so
determinism is untouched. See D-012.

---

## 3. The tank: world model

Decided in Q2 (D-003): **fully aquatic, with a hostile but reachable surface.**

### 3.1 Geometry

- A long, rectangular aquarium. Simulation space is continuous coordinates in a box **512 × 256 × 192 units**
  (L × W × H) [tune]. A sapient body is about 1–2 units long, so the tank is roughly "250 bodies long."
- **Glass walls** on four sides. They are visible to inhabitants: reflections, refracted light and the dim
  shapes of the room beyond. The glass is a hard, unexplainable boundary, which is the seed for watcher
  awareness (§10.6).
- **Terrain:** a heightfield at 1-unit resolution (512 × 256), plus volumetric features (rock arches,
  caves, overhangs) stored as a sparse list of solids. Substrate types: sand, silt, gravel, rock, clay
  and coral/reef (grows biologically).
- **Hydrothermal vents:** 1–4 per tank, placed by seed. They are local sources of heat, minerals and
  chemical energy, and the cradle of primordial life. Vents can go dormant or erupt (§12).
- **Surface layer:** the top ~8 units are the air/water interface. Above it is air: hostile (desiccation,
  no buoyancy) but reachable. Breaking the surface is a milestone, not a feature.
- **The lid and the Lamp:** the light source above the tank. The Lamp's cycle defines days. Its brightness
  and color shift with seasons (§4).

### 3.2 Physical fields

Coarse 3-D grid at **4-unit resolution → 128 × 64 × 48 = 393,216 cells** [tune]. Each cell stores:

| Field | Units | Driven by |
|---|---|---|
| `light` | 0–1 | Lamp intensity × depth attenuation × turbidity × shading by plants/structures |
| `temperature` | °C (tank scale ~4–32) | Climate driver (§4), vents, surface exchange, diffusion |
| `oxygen` | 0–1 | Photosynthesis, respiration, surface exchange, decay |
| `nutrients` (N, P, minerals) | mg-ish | Decay, vents, sediment release, runoff pulses (§4) |
| `organic` (detritus) | mass | Death, waste |
| `toxin` | 0–1 | Pollution from activity (smelting, middens, overcrowding), some organisms |
| `turbidity` | 0–1 | Sediment suspension, plankton blooms |
| `current` | vec3 | Base circulation + thermal convection + surface wind stress (§4) |

Fields update every N ticks (diffusion, advection and reactions with operator splitting) [tune]. Each cell
also holds a **strain table** for population-level life (§5.2).

### 3.3 Resources and environmental feedback

- **Primary producers:** algae mats, phytoplankton (in the field) and macro-plants (kelp/reed analogs, as
  individual entities). Growth depends on light, nutrients and temperature.
- **Regeneration vs. depletion:** each resource has a regrowth curve. Overharvesting pushes stocks below
  their recovery threshold, and they collapse (Allee effect).
- **Pollution:** toxin accumulates from activity, lowers growth and raises disease. Currents spread it.
  Sediment slowly sequesters it.
- **Population pressure:** middens raise organic load → oxygen drops → dead zones.

All parameters live in `config/world/*.toml` with named presets (`gentle`, `default`, `harsh`) (P§6).

---

## 4. Climate: the Midwest record and tank weather

Two independent layers (D-017):

### 4.1 Layer 1: historical Midwest climate drives the tank (simulation input)

- **Dataset:** NOAA GHCN-Daily records for Chicago (Midway and O'Hare stations; public domain),
  pre-processed **at development time** into a bundled, versioned, checksummed file
  (`data/climate/chicago_daily.bin`). The simulation never touches the network.
- **Mapping:** **one tank cycle replays one real Midwest year.** The 365 real days are resampled to 48
  tank days, with each tank day blending about 7.6 real days. Extremes are preserved rather than averaged
  away. For each tank day:

| Real variable | Tank effect |
|---|---|
| Mean temperature | Target surface temperature (linear map, e.g. −20 °F…100 °F → 4…32 °C), with thermal inertia increasing with depth |
| Day length (from date + latitude 41.9° N) | Lamp on/off duration within the 1,024-tick day |
| Cloud cover proxy (from precip and temperature range) | Lamp brightness |
| Precipitation | **Runoff pulses:** nutrient and turbidity input at the surface. Big storms bring algae blooms. |
| Wind (where recorded) | Surface current stress, mixing depth |
| Snow cover / freezing days | Surface ice film: blocks surface exchange, oxygen drops (winterkill risk) |

- **Which years?** The seed draws a **shuffled block sequence of real years** (blocks of 3–10 consecutive
  years, so multi-year droughts stay intact). Different tanks live through different Midwest histories.
  The Chronicle can mention it in passing ("this tank's 88th cycle mirrored the drought of 1988"), and
  the Hall of Fallen Tanks records which real years a tank lived through.
- **Harshness knob:** `climate.amplitude` scales anomalies around the mean, so `harsh` worlds get
  exaggerated real extremes.
- **Determinism:** the dataset is part of the determinism contract.

### 4.2 Layer 2: live outside weather (broadcast flavor only)

- The hub (not the simulation) polls the **National Weather Service API** (`api.weather.gov`, free, no key)
  for current Chicago conditions, about every 30 min, with caching.
- It is used **only** on screen: "Outside the tank room: Chicago, 34 °F, light snow." TNN's weather
  person can contrast it with tank weather.
- **It never reaches the simulation.** If the network is down, the widget hides itself.

### 4.3 Tank weather forecasting (pitch accepted)

- A **forecaster** in the hub (broadcast side, read-only) predicts tank conditions 1–5 days ahead.
- It is deliberately *imperfect*: a statistical model trained only on the tank's own observed history
  (seasonal climatology + persistence + recent anomaly regression). It does **not** peek at the lookahead
  buffer or the bundled dataset's future.
- Every forecast is logged and scored. The weather person has a **real accuracy record** ("our forecast
  has been wrong three days running"). It is grounded and checkable, and the personality material writes itself.
- Inhabitants have their own weather beliefs (§7.6), and the broadcast can compare them: "The Reed-Folk
  are stockpiling for an early cold. Our models disagree."

---

## 5. Levels of simulation and how each era emerges

The central problem (P§4): chemistry → civilization on a Mac mini. The answer is **one world, several
levels of detail, with promotion by evolved traits rather than timers** (D-004).

### 5.1 The super-individual principle

Every living thing is an `Agent` with a **multiplicity** `m`, the number of real organisms it stands for.

- Microbial life: strains in field cells (implicit, `m` in the millions).
- Small organisms: super-individuals (`m` = 10–10,000) with one position and behavior.
- Larger organisms: individuals (`m` = 1), cheap behavior.
- Social animals: individuals with partial minds.
- Sapients: individuals with full minds (always `m` = 1).

Super-individuals **split** when they need more resolution (near the camera, near sapients, during
predation that matters) and **merge** when they don't. This keeps the agent count bounded however large
the real population gets (Scheffer et al.'s super-individual method, widely used in ecology models).

### 5.2 Tier 0–1: Primordial chemistry and microbial strains

- **Tier 0 (prebiotic):** vent chemistry in the fields. An energy gradient (vent minerals + heat, or light)
  feeds a small reaction network. **Proto-replicators** appear stochastically where energy and precursors
  exceed a threshold. The probability is configurable, and some tanks never get life (*a valid ending*).
- **Tier 1 (strains):** each field cell holds up to K (≈ 8) **strain entries** `(strain_id, count)`. A global
  **strain registry** stores genomes. Per update:
  - Growth/death from metabolism genes vs. local fields (logistic with resource uptake).
  - Mutation: new strains spawn with probability ∝ `count × mutation_rate`.
  - Transport by currents and diffusion; predation between strains (phage/grazer genes).
  - Bounded: at most ~4,096 live strains globally [tune]. The smallest and least distinct are merged or pruned.
- **Speciation** is measured, not declared. Strains are clustered by genome distance + reproductive
  compatibility into **species** every N cycles. Mass extinctions show up as cluster loss.

### 5.3 Tier 2: Organisms

- Promotion: a strain whose **multicellularity** and **body size** genes cross thresholds gets instantiated as
  super-individual agents drawn from its population count.
- Agents have a body plan (from genome → also drives the procedural creature mesh), metabolism and senses.
  Behavior comes from a **tiny evolved policy**: a weighted reflex/utility table over ~10 drives × ~12
  actions (approach, flee, eat, graze, rest, school, mate, guard, hide, wander, attack, follow). This is
  cheap enough for 10⁴–10⁵ agents.
- Sexual reproduction, predation, symbiosis (paired genes for host/symbiont benefit), parental care
  (a gene) all evolve.

### 5.4 Tier 3: Social animals: "one mind, many capacities"

Key design choice (D-005): **there is one Mind module, and its richness is a continuous function of the
evolved `neural_capacity` gene.** The same code that runs a sapient also runs a clever fish with most
features dialed down or off.

| Capacity | Memory slots | Known individuals | Prediction rules | Planner depth | Communication |
|---|---|---|---|---|---|
| < 0.15 | — (reflex policy only, Tier 2) | — | — | — | — |
| 0.15–0.35 | 4 episodic | kin only (scent) | 4 | 0 | innate alarm/food/mating signals |
| 0.35–0.55 | 16 | ~8 (+kin) | 16 | 1 | learned signals (few meanings) |
| 0.55–0.75 | 48 | ~30 | 48 | 2–3 | proto-lexicon (≤ 50 words) |
| > 0.75 (sapient) | 64–128 | 50–150 | 128 | 4–6 | full lexicon, grammar tiers, teaching |

Thresholds are soft: features fade in, and all values come from config [tune].

**Intelligence must pay for itself.** `neural_capacity` costs metabolic energy, lengthens juvenile
dependency (offspring need care) and slows reproduction. It is selected for only where the environment is
**variable** (seasons, Midwest anomalies, shifting resources), **socially competitive** (group living,
reputation, deception) and **manipulable** (organisms with manipulators can exploit tools). This is
the honest bet of the project: sapience appears only when the tank makes it worthwhile.

### 5.5 Tier 4: Sapients (full minds, §7)

Promotion to full sapient treatment is per individual by capacity. Viewers see it per species: the
overlay names a species "sapient" when its median capacity has stayed > 0.75 for N generations.

**Body-plan gate (Q3, D-007):** a mind can **build** only if its body has the `manipulator` trait
(fins/tentacles with grip > threshold), and it reads well on screen only with an `expressive_face`
(eyes + brow/mouth analog). Both are evolvable genes with fitness value of their own (grip helps
feeding; expressive faces help signaling). A smart species without hands is a valid, poignant outcome:
it can be social and cultural, but it can't build.

### 5.6 How each era emerges (summary)

Eras are **labels the observer computes**, never states the simulation enters.

| Era label | Detected when… | What has to emerge first |
|---|---|---|
| Primordial | Life absent or only Tier-1 strains | Replicators from vent/light chemistry |
| Biological | Tier-2 organisms present | Multicellularity + body size genes |
| Social | ≥ 1 species with stable groups (detected §8.3) and learned signals | Parental care + group-living payoff + capacity ≥ 0.35 |
| Cultural | Learned behavior transmitted non-genetically for ≥ 3 generations (tool use, names, territories) | Capacity ≥ 0.55, social learning, manipulators |
| Civilizational | ≥ 1 settlement (§10.4) with specialization + an institution | Storage, construction, discoveries, population density |
| Ending | §21 | — |

Eras can go **backward** (a civilization collapses to cultural, or a species loses sapience over
generations). The overlay shows the *highest era currently present* and per-species status.

---

## 6. Genomes

Fixed-layout float vectors with named loci, grouped by module (D-008). They are simple, fast, easy to mutate
and easy to diff and visualize.

```rust
struct Genome {
    // ~96 loci [tune]; all f32 in [0,1] unless noted, mapped by expression functions
    body:        BodyGenes,        // size, shape params (length/width/height ratios), segment count,
                                   // fin/limb count & type, manipulator_grip, armor, coloration (3 hues + pattern),
                                   // expressive_face, bioluminescence, hard_parts (fossilization chance)
    metabolism:  MetabolismGenes,  // diet axis (photo/chemo/herbi/omni/carni), efficiency, temp optimum & tolerance,
                                   // O2 need, toxin tolerance, max age, growth rate
    senses:      SenseGenes,       // vision range & acuity, light sensitivity, chemo sense, lateral line, hearing
    reproduction:ReproGenes,       // sexual?, clutch size, juvenile period, parental_care, mating_signature (3 loci)
    behavior:    BehaviorGenes,    // reflex/utility weights for Tier-2 policy (~10×12 compressed via 24 loci)
    cognition:   CognitionGenes,   // neural_capacity, learning_rate, memory_decay, social_learning_bias, play_drive
    social:      SocialGenes,      // sociality, kin_bias, dominance, cooperation_bias, signal_innate_set
    personality: PersonalityGenes, // baselines: boldness, sociability, curiosity, aggression, patience, conformity
    voice:       VoiceGenes,       // pitch, timbre, articulation range → spoken sound (§9.4)
}
```

- **Mutation:** Gaussian per locus with rate/size from config, plus rare **duplication** (copy one module's
  sub-block into a related slot, a cheap stand-in for gene duplication driving novelty).
- **Recombination:** uniform crossover by module once sexual reproduction evolves.
- **Reproductive isolation:** `mating_signature` distance gates mating success. This drives speciation.
- **Expression:** the phenotype is computed once at birth (body parameters, mind capacity budget) and cached.
- **Interface for Phase 1:** Phase 1 creates sapients from a *hand-seeded* genome distribution. Phase 2
  replaces the seeding with evolution through the same `Genome` type.

---

## 7. Minds

The heart of the project (P§5). Minds are deterministic Rust code with bounded memory. No LLM.

### 7.1 The think loop

Minds are **event-driven**, not ticked every tick. A mind re-decides when:
1. its current action completes,
2. an **interrupt** fires (threat percept above fear threshold, being addressed, pain, a large surprise), or
3. a heartbeat timer expires (about every 32 ticks).

On average that is about 150 decisions per mind per day [tune]. Movement between decisions is a path the
renderer interpolates.

```
perceive → appraise (emotions, surprise vs predictions) → update memory/beliefs/ToM
        → (re)score candidate actions & active plan → act → write ThoughtRecord (if meaningful)
```

**Cost target:** ≤ 15 µs per decision for a sapient, single-threaded. Minds are processed in parallel by
spatial region with deterministic merge order (§18.4).

### 7.2 Perception

- Spatial hash query within genome-set sense ranges. Vision is attenuated by light and turbidity, chemo
  sense works downstream in the current, and the lateral line picks up nearby movement.
- **Noise:** misidentification probability rises with distance and darkness, so they can mistake a rock
  for a predator. **No global knowledge,** ever. Everything not perceived comes from memory or communication.
- Output: `Percept { kind, entity_id?, est_position, confidence, salience }`, up to 16 per decision.

### 7.3 Drives

Eight drives in [0,1], each with a genetic weight and a personality modulation:

`hunger · safety · rest · belonging · status · curiosity · reproduction · care_for_kin`

They rise over time or with events and fall when satisfied. Drive urgency feeds action utility.

### 7.4 Personality

Six traits, heritable baselines with **experience drift** (e.g. surviving many dangers shifts boldness,
betrayals lower sociability). Drift is small per event and bounded per life:

`boldness · sociability · curiosity · aggression · patience · conformity`

### 7.5 Emotion (appraisal)

Simplified OCC-style appraisal. Each event is evaluated for **desirability** (vs. drives), **expectedness**
(vs. predictions, so surprise amplifies), **agency** (self, known other, unknown, environment) and
**controllability**.

Emotions: `fear · anger · grief · joy · trust · shame · surprise · disgust`. Each is an intensity in [0,1]
with exponential decay (personality-modulated). They bias decisions: fear raises flee/hide utilities,
anger raises confront/punish, grief raises withdraw and boosts memory salience, and so on. They also tag memories.

### 7.6 Memory and beliefs

**Episodic** (bounded ring, capacity by tier):
```rust
struct Episode { tick, kind: EventKind, place: PlaceRef, actors: SmallVec<AgentId,4>,
                 outcome: OutcomeTag, emotion: EmotionVec, salience: f32 }
```
Salience = emotional intensity × surprise × personal stakes. Low-salience episodes are evicted first.
High-salience ones resist decay ("the day the east shallows took my sister").

**Semantic beliefs** (bounded store, capacity by tier):
```rust
struct Belief { subject: Concept, relation: Relation, object: Concept|Value,
                confidence: f32, source: Source /* experience|told_by(AgentId)|inferred|tradition */,
                formed_tick, last_confirmed_tick }
```
- *Concepts* come from a shared ontology: places, species, individuals, objects, actions, seasons, light
  states, weather states, discovered things, and abstract nodes unlocked by culture (spirits, the Lamp,
  the glass, the ancients).
- Beliefs form by **consolidating** repeated episodes, by **being told** (§9) and by **inference** from
  prediction rules. They can be wrong and they can persist when wrong (confirmation bias: disconfirming
  evidence is discounted by `conformity` and `confidence`).
- Weather beliefs ("cold follows the dimming light") come from the same mechanism, which makes the TNN
  comparison in §4.3 possible.

### 7.7 Prediction (anticipation)

The core of "acting before the outcome arrives."

- **Contingency learning.** A mind tracks counts of `(cue → outcome)` within a time window, where cues come from
  a fixed feature vocabulary: place, season phase, Lamp state, temperature trend, presence of a species or
  individual, own or others' actions, discovered objects. It computes ΔP = P(outcome | cue) − P(outcome | ¬cue).
  Rules with |ΔP| above threshold and enough evidence become **PredictionRules** with confidence.
- Small samples produce superstitions, and that is intended. Rules spread socially as beliefs.
- **Use in decisions:** each candidate action is scored with the expected value of outcomes predicted by
  active rules, so stockpiling before scarcity, fleeing before the predator is near, allying before a threat and
  striking first out of fear come from the same machinery.
- Bounded rule count, evicting the least-used rules.

### 7.8 Theory of mind

For each known individual (Dunbar cap by tier):
```rust
struct Model { who: AgentId, trust: f32, affinity: f32, fear: f32, respect: f32,
               ledger: i16 /* favors owed (+) or owed to me (−) */, grudges: SmallVec<EpisodeRef,2>,
               est_traits: PersonalityEstimate, predicted_intent: IntentGuess, last_seen: Tick,
               believed_roles: RoleBits }
```
Updated from direct observation, interactions and gossip (weighted by trust in the source). Predicted
intent uses the observer's own rules applied to *their estimate* of the other's situation, so minds can
be wrong about each other.

### 7.9 Decisions and plans

- **Utility AI** over ~40 action types (eat, gather, store, rest, flee, hide, approach, follow, attack, defend,
  groom, share, ask_help, offer_help, propose_task, accept, refuse, teach, imitate, signal/speak, gossip,
  court, mate, care, explore, play, experiment, build, repair, trade, mourn, punish, migrate, dig, …).
  Each action is scored by a handful of **considerations** (drive relief, predicted outcomes, emotion bias,
  personality, social norms the mind believes, current plan coherence). Scores get small deterministic noise
  from the agent's RNG stream so individuals don't move in lockstep.
- **Plans:** a small **HTN library** of multi-step tasks (gather-and-store, build shelter, cooperative hunt,
  teach skill, migrate, raid, hold ceremony…). Plans are interrupted by higher-utility events and
  revised. Planner depth is capped by tier.
- **Norms:** beliefs of the form `(action, context) → sanction` become extra considerations. Norms
  emerge from punishment episodes and spread like other beliefs (the root of law, §10.4).

### 7.10 Social learning

- **Imitation:** a mind that observes another's action → good outcome may adopt the behavior with probability
  ∝ observed success × model's prestige × learner `social_learning_bias` × `conformity`.
- **Teaching:** a deliberate action. Kin teach skills and beliefs at a higher fidelity than imitation.
- **Rumor:** beliefs pass through gossip with trust-weighted acceptance and a small **distortion**
  chance (subject or object substituted with a related concept). False rumors are possible and expected.

### 7.11 Life stories and notable individuals

- Each sapient keeps a compact **Life Record**: birth (parents, place), name (§9.4), key episodes (top-salience),
  relationships, roles held, discoveries, children, death (cause, place, mourners).
- **Notability score**, updated continuously = impact on others (number and weight of beliefs and memories that
  reference them), discoveries originated, group founding, kills/saves, role prestige, being a last survivor.
  Above-threshold individuals are flagged *notable* and their thought logs are kept at full resolution.

### 7.12 Thought log

Every *meaningful* decision (plan change, interrupt, social action, high emotion, discovery) emits:
```rust
struct ThoughtRecord {
    tick, agent: AgentId,
    perceived: SmallVec<PerceptSummary, 4>,
    felt:      EmotionTop2,
    expected:  SmallVec<(RuleId, Outcome, conf), 2>,
    drives:    DriveTop3,
    chose:     ActionOrPlan,
    because:   SmallVec<Consideration, 3>,   // top-weighted considerations
    target:    Option<AgentId|Place|Object>,
}
```
It renders to compact text for logs and LLM prompts:
`worried(food_store=low) + expects(winter, 0.8) → plan: gather_extra; ask_help(Kesh, trust 0.7)`

**Storage:** full resolution for notable and spotlighted agents. Ordinary agents keep a ring buffer that is
promoted to permanent if they later become notable. Logs roll into compressed daily segments (§18.5).

---

## 8. Cooperation, groups and roles

### 8.1 Joint action protocol

`propose(task, partners, terms)` → each partner evaluates (utility of task × trust in proposer × ledger ×
personality) → `accept | refuse | counter` → execute → **outcome is remembered** and updates trust and ledger
for everyone involved. Free-riders get observed, gossiped about and eventually excluded.

Task types: cooperative hunt/gather, build, defend, teach, care for young, raid, migrate, ceremony.

### 8.2 Division of labor

Specialization comes from comparative advantage: skill improves with practice (learning curve), and
asking a skilled other is cheaper than doing it yourself once trust exists. Labor division
appears in measured activity histograms. It is never assigned.

### 8.3 Groups (detected, then believed)

- **Detection:** every N days the observer runs deterministic community detection (weighted label
  propagation) on the affinity + co-location + cooperation graph. Detected clusters are the
  *observer's* groups: for metrics and overlays.
- **Belief:** when members repeatedly act together and use a shared name (§9), they form the belief
  `self ∈ GroupX`. *Then* in-group effects kick in (trust bonus, defense, norms). Groups are therefore both
  measured and real to their members, and the two can disagree (an outcast who still believes they belong).
- Splits, merges and feuds emerge from changing affinity and leadership contests. The observer logs them as events.

### 8.4 Roles (emergent labels)

A role is **(a)** a behavior pattern the observer detects (gatherer, guard, teacher, healer, leader, outcast,
prophet, trader, builder, storyteller…) via activity histograms + reputation, and **(b)** a **social belief**
others hold ("Kesh heals"). (b) changes behavior: the sick seek Kesh. No code assigns a role.
Leadership emerges from prestige (others imitate and defer) and dominance (others fear), so you can
get respected leaders, feared tyrants, or both.

---

## 9. Language: signals, lexicons and spoken sound

Tracked explicitly because First Contact depends on it (P§5.2, §17).

### 9.1 Stages

1. **Innate signals** (genetic): alarm, food, mating, distress. Fixed meanings.
2. **Learned signals:** associations between a vocalization and a context, learned from others.
3. **Lexicon:** a **naming game** (Steels-style). Speakers name a concept, hearers adopt or align, and
   success reinforces the word. Shared vocabularies emerge within interacting groups and **drift** apart
   between separated groups.
4. **Grammar tiers:** 0 single words · 1 two-word combinations · 2 consistent word order (order
   parameters are chosen by early convention and drift) · 3 compositional (modifiers, tense/aspect markers) ·
   4 abstract and recursive (beliefs about beliefs, hypotheticals, narrative). Tier advances require
   capacity + lexicon size + successful-communication rates.

### 9.2 Data model
```rust
struct Lexicon { entries: SmallMap<Concept, SmallVec<(WordId, score: f32), 2>> }   // per mind
struct Word    { form: PhonemeString, culture: CultureId, coined_tick, coined_by: AgentId }
struct LanguageState {  // per detected speech community
    phonology: Phonology, grammar_tier: u8, word_order: WordOrder,
    shared_lexicon_size: u16, coherence: f32 /* agreement rate within community */,
    abstract_concepts: u16, parent: Option<LanguageId>, split_tick: Option<Tick>,
}
```

### 9.3 Measurements
- Lexicon size, within-community **coherence**, grammar tier, abstract-concept count.
- **Mutual intelligibility** between communities = shared form–meaning pairs (after sound-change
  alignment) / union.
- A **language family tree** (splits recorded when intelligibility drops below a threshold).
- **True language** (First Contact gate, §17): grammar tier ≥ 3, coherence ≥ 0.7, shared lexicon ≥ 200,
  abstract concepts ≥ 20, sustained for ≥ 2 generations [tune].

### 9.4 Spoken sound (pitch accepted)

So you can **hear** the cultures.

- **Phonology genome per culture** (inherited from the parent language, drifts over time):
  - Phoneme inventory: a subset of a master inventory of ~40 consonants and ~12 vowels, constrained by
    the species' `VoiceGenes` (a species without "lips" can't make bilabials).
  - Syllable templates (`CV`, `CVC`, `V`, `CCV`…) with weights, a stress rule, vowel harmony on/off,
    reduplication tendency, preferred word length.
- **Word coining:** when a concept needs a new word, it is generated from the phonology + the coiner's RNG,
  sometimes by **compounding** existing words ("dim" + "water" → word for "winter").
- **Sound change:** at intervals, each language applies a randomly drawn **regular sound law** (e.g.
  intervocalic /p/ → /f/, final vowel loss) to its whole lexicon. Split languages diverge in a systematic,
  linguistically plausible way, so cognates stay recognizable for a while.
- **Names:** personal and place names are built from the culture's own words ("Kesh-ta-vor" = "born at the
  bright vent"), so they mean something, and the Chronicle can gloss them.
- **Audible speech:** the renderer/hub sends `PhonemeString`s through **espeak-ng in phoneme-input mode**
  (no real-language words), shaped by species voice genes (pitch from body size, formant shift, a
  water/bubble filter). It is used for ambient murmur in aquarium mode, subtitled lines in spotlight, and
  First Contact interviews where the inhabitant speaks their language and the pod's "translator" renders
  only what maps (§17). Untranslatable concepts are left in the native word.
- **Archive:** each tank's languages go to the Hall of Fallen Tanks with a sample lexicon and audio.

---

## 10. Discovery, knowledge and culture

Phase 3 builds this. The data models are fixed now so Phase 1 minds can hold knowledge from the start.

### 10.1 Discovery graph

A hand-authored graph of **~150–250 nodes** [tune], *described compositionally*:

```rust
struct DiscoveryNode {
    id, kind: Tool|Structure|Technique|Belief|Institution|Practice,
    function: FunctionTag,          // cut, pierce, contain, bind, shelter, light, heat, propel, carry, record,
                                    // measure, heal, govern, exchange, worship, ...
    requires: Prereqs {
        materials: Vec<MaterialClass>,      // must be locally accessible/known
        knowledge: Vec<NodeId>,             // must be known by the experimenter (or a teammate)
        min_group: u16, min_capacity: f32,
        needs: Vec<NeedSignal>,             // e.g. stored-food losses, predation pressure, cold deaths
    },
    difficulty: f32, forms: Vec<FormTag>, sizes: SizeRange,
}
```

**Aquatic graph (Q2):** written from scratch for water. Some sample chains:

- *Hard tools:* shell edge → stone/shell blade → hafted spear → barbed harpoon → vent-forged metal blade
- *Fiber:* kelp cordage → woven net → woven basket → sails for current-riding → rope bridges between reefs
- *Food:* caching → algae gardens → shellfish beds → **reef cultivation** (agriculture analog) → selective
  breeding of farmed species
- *Light & heat:* vent cooking → **vent smithing** (metals from mineral chimneys) → **glowlamps**
  (domesticated bioluminescent organisms) → lamp-lit night activity
- *Building:* sand burrow → stacked-stone wall → woven reed hall → coral-mortar masonry → stilted/
  floating structures → reef cities
- *Movement:* current-reading → current mills → **air bladders** (gas collection) → surface expeditions
- *Record:* tally scratches → knotted cords → glyph carving → writing → archives
- *Thought & institution:* burial → ancestor rites → **Lamp-watching** (astronomy analog, from the light
  cycle and wall reflections) → calendar → priesthood → law → shell money → markets → schools → medicine
- *Watcher beliefs:* see §10.6

Several nodes have **alternative routes**, and nothing is required. A culture can skip writing and keep
oral law, or never smith metal.

### 10.2 How discoveries happen

No research points. Discovery is a mind's action:

1. **Triggering:** an `experiment` or `play` action is chosen through normal utility: curiosity drive, play drive
   in juveniles, or a **need** (repeated failure tied to a NeedSignal, e.g. losing stores to scavengers).
2. **Candidate:** the mind samples a recipe `(function, available material, known technique, form)` biased
   toward its current need and recent observations. Observing other species or fossils can bias it (§11).
3. **Outcome:** success probability = f(node difficulty, skill, capacity, luck). Failure teaches a little
   (raises later success) and can create false beliefs ("the red stone is cursed").
4. **Success:** the mind gains a `KnowledgeItem`. A Registry entry and an event are emitted.

**Variants:** material/form substitutions outside the node's canonical list produce a **variant
descriptor** (shell spear, stilt temple). Variants get their own Registry entries and asset requests.

### 10.3 Knowledge lives in minds

```rust
struct KnowledgeItem { node: NodeId, variant: Option<VariantId>, proficiency: f32,
                       learned_from: Source, learned_tick }
```
- It spreads by teaching, imitation, trade contact, migration and conquest (captured or absorbed people
  bring knowledge).
- **Institutions** hold knowledge by keeping roles filled (a temple trains successors) and, after
  writing, in **records** (artifact objects that can be read by the literate).
- **Loss:** when the last holder dies and no readable record exists, the node is **lost** for that culture.
  The Registry marks it. Rediscovery is possible and is logged as such (dark ages are real).

### 10.4 Settlements, economy, power and conflict (Phase 3 detail)

- **Settlement:** detected when a group repeatedly returns to a place, stores goods there and builds
  structures. Settlements grow, specialize, get abandoned and decay into ruins (§11).
- **Trade:** exchange actions between agents/groups with *believed* values. Shell money becomes possible
  once a medium of exchange is widely trusted.
- **Leadership & law:** norms (§7.9) become formal law when a leader or institution enforces them and
  (later) records them.
- **Belief systems:** shared beliefs about unexplained causes (the Lamp, vents, death, the glass) gather into
  doctrines via the normal belief machinery plus prophet roles. Ritual comes from repeated group actions
  tied to those beliefs.
- **Conflict:** raids and wars come from resource competition + grudges + *predicted* threats (fear-driven
  first strikes). Conquest moves people, knowledge and style.
- **Collapse dynamics:** feedbacks such as overharvest → famine → norm breakdown → violence → knowledge loss.
  Nothing is scripted; these are measured afterwards.

### 10.5 Style genome

```rust
struct StyleGenome {
    material_palette: [MaterialClass; 3], color_palette: [Hsl; 4], silhouette: f32 /* low&heavy ↔ tall&light */,
    roof_form: FormTag, symmetry: f32, ornament: f32, curvature: f32, verticality: f32,
    motif: MotifId /* sacred shapes from beliefs */, wear_tolerance: f32, scale_bias: f32,
}
```
- Initialized from environment and materials (reed cultures: light and tall; stone cultures: heavy and low).
- Drifts by: beliefs (sacred colors and shapes), wealth (ornament ↑), hardship (austerity), status display,
  isolation (random drift) and contact (borrow, blend or **reject**, where rejection pushes the style
  away from the rival's).
- **Visibly changes** the renderer's output for every structure and object (§15).

### 10.6 Watcher awareness (very rare)

The world seeds the conditions and the minds do the rest:
- **The glass:** an unexplainable boundary that reflects, and minds that touch it remember it.
- **The Lamp:** cyclical; its predictability invites calendar-making and worship.
- **Glimpses:** rare perceptual events. A large shadow passes the glass (the room beyond), or the
  observation pod catches the light (§17). Each is a low-probability percept for nearby minds.
- These feed ordinary belief formation. A culture *may* end up with a doctrine about Watchers. The
  observer flags it when such beliefs are held by > X% of a culture. Target frequency: well under 5% of
  civilized tanks [tune].

---

## 11. Fossils, strata and ruins

(pitch accepted · D-018)

### 11.1 Sediment and burial
- Every terrain column has a **sediment deposition rate** (fields: turbidity settling, organic fall,
  currents). Sediment accumulates as **strata layers** (thin records: tick range, dominant material,
  color, notable inclusions).
- When an organism dies, a **fossilization roll** depends on `hard_parts`, substrate and burial speed.
  Success creates a `Remains` record with a genome snapshot and position, buried as sediment accrues.
- Structures that are abandoned **decay → ruin → buried ruin**. Objects left behind become **artifacts**.

```rust
struct Buried { id, kind: Fossil|Ruin|Artifact|Record, descriptor: DescriptorRef,
                origin: (TankCycle, SpeciesId|CultureId), pos: Vec3, depth: f32, condition: f32 }
```
**Bounded:** at most ~20k buried items per tank. Deeper and older ones merge into "fossil beds," and the
most notable are always kept.

### 11.2 Discovery of the past
- `dig` and `explore` actions can uncover buried items. Erosion (currents, vent eruptions) can expose them too.
- Minds form beliefs about finds through normal machinery: "the ancients," monsters, gods, ancestors, or
  correct inference if the culture has the relevant knowledge.
- Finding a lost civilization's artifact **biases experimentation** toward that node (big boost to the
  candidate sampling weight), so ruins can end dark ages. Rediscovery events cite the source find.
- Fossils of extinct species can enter mythology ("the great armored ones before the Lamp").

### 11.3 On screen
- **Cutaway shot:** the camera can drop to a cross-section view showing strata, fossils and buried ruins
  under a living city.
- Filler segment: **"From the Strata"** (fossil of the week, deep-history recap).
- Hall of Fallen Tanks includes the stratigraphic column of each ended tank.

---

## 12. The storyteller

RimWorld-style pacing, *without* targeting (P§6).

- **Event catalog** (all physical, all inputs to systems): meteor/debris falling through the lid, vent
  eruption or dormancy, current shift, algae bloom (also climate-driven), cold snap / heat spike
  (amplifying the Midwest driver), **mutation burst** (temporary higher mutation rate), **pathogen
  emergence** (a new Tier-1 strain with a parasitic genome, which then evolves on its own: plague is an
  organism, not a status effect), lamp flicker, silt slide.
- **Pacing:** a tension variable is computed from *global physical and statistical* state only (recent event
  intensity, total biomass variance, time since last major event). The storyteller lowers the chance of
  events right after big ones and raises it after long calm, following a configurable curve.
- **No targeting.** Event location is drawn from a physical distribution (geology, current paths) using
  the storyteller's own RNG stream. It never reads where populations or groups are. Enforced by
  architecture: the storyteller module gets a read-only `PhysicalView` with no agent data.
- Configurable presets and per-event rates in `config/storyteller/*.toml`.

---

## 13. Events, history and the Discovery Registry

### 13.1 Event model
```rust
struct Event {
    id: EventId, tick: Tick, kind: EventKind,     // ~120 kinds: Birth, Death, Discovery, GroupFormed, Split, War,
                                                  // SettlementFounded, KnowledgeLost, SpeciesExtinct, FirstWord, ...
    actors: SmallVec<EntityRef, 4>, place: PlaceRef, culture: Option<CultureId>,
    data: EventPayload,                           // kind-specific struct
    importance: f32,                              // computed by sim (stakes) — §14
    novelty_key: u64,                             // for novelty scoring (kind × subject hash)
    cause: SmallVec<EventId, 2>,                  // causal links where known (famine → migration)
}
```
- Written by the simulation to an **append-only segment log** (length-prefixed binary, zstd, rotated
  daily). The hub indexes it into **SQLite** for queries (news, chronicle, archives).
- **Causal links** are what make grounded narration possible ("because the harvest failed…" is only
  allowed when the link exists).

### 13.2 Discovery Registry
```rust
struct RegistryEntry {
    key: DescriptorKey,   // node + variant hash
    descriptor: Descriptor { function, materials, form, size, style_snapshot },
    first: (Tick, CultureId, AgentId), spread: Vec<(Tick, CultureId, via: Teach|Trade|Migration|Conquest|Independent)>,
    status: Active|Lost(Tick)|Rediscovered(Tick), peak_users: u32, instances_built: u32,
}
```
Exported per tank as JSON. It feeds the Asset Request Queue (§15.3) and cross-seed rankings (§15.4).

---

## 14. Interestingness, time control and the camera

### 14.1 Interestingness score (per event)
```
I(e) = w_n·novelty + w_s·stakes + w_c·conflict + w_e·emotion + w_p·people + w_v·visibility
```
- **novelty:** −log frequency of `novelty_key` in this tank (and across tanks for "firsts ever"). A first fire scores enormous, the 400th birth scores nothing.
- **stakes:** lives, minds, knowledge or settlements at risk or lost.
- **conflict:** hostile actions, contested leadership, war.
- **emotion:** sum of emotion intensity in minds within perception range (a funeral with 40 grieving).
- **people:** notability of involved individuals.
- **visibility:** whether it produces a watchable state change (construction, crowd, destruction).

The **scene interest** at time t is a decayed sum of event scores in a region. **Global interest** is the max over regions.

### 14.2 Speed controller
- Uses the **lookahead buffer** (§2.3): it sees interest up to 60–180 s ahead in wall time.
- Maps forecast interest → target playback speed (log scale, with era-specific caps), with smoothing
  and hysteresis so it never oscillates. Speed ramps down *before* a peak and holds through it.
- The overlay always shows the current speed ("▶▶ 1 day / 30 s").

### 14.3 Camera Director
- **Aquarium mode** (default): scored random walks among "beauty anchors" (light shafts, plant forests,
  vents, glass edges, settlements from afar, wandering creatures). Long shots (20–60 s), slow dollies.
- **Event mode:** when scene interest crosses a threshold, the Director plans a shot sequence: establish →
  approach → hold on subject → reaction/crowd → pull back. Minimum hold times, no cuts during a hold, and
  crossfade or dolly transitions only.
- **Shot language** includes construction timelapse (structures rise piece by piece), the strata cutaway (§11),
  spotlight follow (one notable individual with their thought subtitle), crowd gather and wide tank.
- **Replay buffer:** directives + state diffs for the last N hours are kept compressed. Clips are re-rendered
  from simulation state when a news package needs them, so this is not stored video and is cheap.
- Directives are a plain stream `{t, mode, target, shot_type, duration, transition}`, so the renderer is swappable.

---

## 15. Renderer and asset system

### 15.1 Renderer
- **three.js** page served locally by the hub, receiving snapshots and diffs + camera directives over WebSocket.
- Low-poly stylized miniature: volumetric light shafts, caustics (projected texture), drifting
  particulates, depth fog by turbidity, glass-edge reflections, bioluminescence at night.
- **Instancing** for creatures/plants (thousands of instances per draw call), LOD by distance.
- HTML/CSS broadcast graphics layered on top (§16).
- Catch-up on reconnect: request snapshot → apply diffs from buffer.

### 15.2 Procedural generator (always available)
- `Descriptor + StyleGenome → Mesh` in < 50 ms. Grammar-based builders per `FunctionTag`/`FormTag`
  (e.g. structure = footprint × wall material × roof form × ornament level × verticality).
- Creature meshes come from body genes (segments, fins, manipulators, face, coloration).
- This means every discovery renders the moment it happens.

### 15.3 Asset Request Queue
- The hub appends `{descriptor, first_instance, culture, usage stats, priority}`. Priority ∝ instances ×
  lifetime × cultures × visibility, decayed if lost quickly.
- A dev session (Claude Code) pulls top requests and writes **finished assets** (Q7, D-009):
  **parametric TypeScript generator modules** (higher-quality builders that still accept style genome
  parameters) and/or small **glTF parts**, plus a thumbnail rendered headless.

### 15.4 Asset Library
- `assets/library/<type>/<asset-id>/` in git: `meta.json` (type, base or variant, parent lineage, origin
  tank/culture/date, style params, usage stats), `generator.ts` and/or `*.glb`, `thumb.png`. Nothing is
  deleted. Revisions are git history.
- **Base assets** are parameterized and the renderer applies the current style genome. **Named variants** are saved
  when a culture's design is distinctive and long-lived, which builds the architectural family tree.
- **Hot-swap:** hub file-watcher → renderer dynamic `import()` of the new module → all instances of that
  descriptor rebuild, with no restart.
- **Pre-launch production (P§8.3):** headless batch runs across hundreds of seeds → ranked descriptor
  frequency/longevity → finished assets for the top N first.

---

## 16. Broadcast, Chronicle and Tank News Network

### 16.1 Always-on overlay
Tank age (cycles, real days), era label, populations by species/culture (small sparkline), current
speed, "Now watching" card (name with gloss, age, role, culture), a timeline ribbon of major events, latest
Chronicle line, and the outside-weather chip (§4.2).

### 16.2 Chronicle
- **Template layer:** each event kind has 3–10 historian-voice templates filled from structured data, so it
  always works.
- **LLM polish (optional):** batches of events → prose paragraph, grounded (§16.4). Shown in a ticker and
  collected per tank.

### 16.3 Tank News Network
- Scheduled at **05:00, 06:00, 12:00, 17:00, 18:00, 22:00 America/Chicago** (DST-aware via IANA tz).
- Segments: top stories, field reports (pod shots), **tank weather** (§4.3 forecast + Midwest context +
  accuracy record), population and economy, obituaries, closing human-interest story.
- **Breaking news** above a high threshold (extinction, first city, first war, collapse, First Contact).
- **Pipeline:** story selection from the event DB → *fact sheet* (each fact with an ID) → LLM writes a script
  as JSON lines `{speaker, text, fact_ids[]}` → **grounding check** → TTS render → timestamped package
  with replay clips → air. Scripts are prepared ≥ 30 min before air time during quiet periods.
- **News team:** simulated characters with persistent personality configs and a running "relationship
  memory" between broadcasts (drafted in Phase 6).

### 16.4 Grounding rules
1. Every sentence must cite ≥ 1 fact ID. Sentences that cite none are dropped (banter lines come from an
   allow-listed template bank instead).
2. All names, numbers, places, dates and causal words ("because", "after") are extracted and must match
   the cited facts (causal words need a causal link in the event data).
3. Failures fall back to template text for that sentence or story.
4. Each aired script is archived with its fact sheet for audit.

### 16.5 Filler segments and Hall of Fallen Tanks
Fallen Tanks recaps, biographies, "This Day in Tank History," architectural family trees, "From the
Strata," "Languages of the Tank" (a short audio piece on a language and its descendants). Archive cards record
lifespan, peak population, civilizations, achievements, signature architecture, notable individuals,
languages, Midwest years lived, stratigraphic column, and cause of death.

---

## 17. Shrink team, Prime Directive and First Contact

- **Observation pod:** a renderer-only object with camera anchoring for field reports. The simulation knows
  of the pod only as a **rare glimpse source** (§10.6): a deterministic, low-probability percept placed
  where the Director had the pod, logged as an input event so replay still works. *(Open question: the
  pod position comes from the Director, which is downstream. See OQ-6.)*
- **Prime Directive:** absolute (D-001 default). No other interaction exists.
- **First Contact gate:** true language (§9.3) **and** a deterministic per-culture probability roll each
  season (very low; config). On success: a `FirstContact` event.
- **Interviews:** the LLM gets *only* the individual's memories, beliefs, lexicon glosses, personality,
  emotions and culture. Every line is checked against that dossier the same way as §16.4. Concepts outside the
  lexicon come back as native words.
- **Bounded belief effect (the single narrative→state bridge):** after contact, the culture forms beliefs
  about the visitors through a **deterministic rule table** keyed on existing state: dominant doctrine type,
  watcher-awareness level, knowledge (writing, Lamp-watching) and the contacted individual's role and personality →
  one of `{gods, demons, ancestors, myth, natural phenomenon, foreigners, science}` with set confidence and
  spread rate. The LLM's interview text never feeds into it.

---

## 18. Architecture and processes

### 18.1 Process map

```
┌────────────────────────── Mac mini M4 (16 GB) ──────────────────────────┐
│                                                                          │
│  [tank-sim]  Rust binary  ──unix socket (framed binary)──►  [tank-hub]  Node/TS
│   • deterministic world                                     • event index (SQLite)
│   • snapshots + event log (disk)                            • lookahead buffer, speed ctl
│   • NO inputs after start                                   • Camera Director
│                                                             • forecaster, NWS poller
│                                                             • content pipeline (chronicle/news)
│                                                             • asset queue + hot-swap watcher
│                                                             • health page (HTTP)
│                                                                │ WebSocket (diffs, directives, overlay)
│                                    [llama-server] ◄─HTTP──────┤
│                                    [tts worker]   ◄─HTTP──────┤
│                                                                ▼
│                                                  [renderer]  three.js page (Chromium/OBS browser source)
│                                                                │
│                                                              [OBS] ──► Twitch + YouTube
│  [supervisor]  launchd jobs + small watchdog (health checks, restarts, disk caps)
└──────────────────────────────────────────────────────────────────────────┘
```

### 18.2 Decoupling
- The simulation never blocks on the hub. Its output channel is bounded, and if the hub is gone it keeps writing to disk
  and drops live frames. The hub reconnects and catches up from disk + the latest snapshot.
- LLM/TTS work is async and queued, scheduled for low-render-load moments (§20.3).
- OBS reads the renderer as a Browser Source, which is isolated from the hub.

### 18.3 Simulation ↔ hub protocol
- Messages: `Hello{version, seed}`, `Snapshot{tick, compressed}`, `Diff{tick, entity deltas}`,
  `Events{batch}`, `Stats{tick, metrics}`. Encoding: bincode/postcard for diffs, MessagePack for events
  (so TS can decode with a small library). Schema is versioned in `protocol/`.
- The renderer protocol (hub → browser) is a separate, simplified schema, so renderer swaps don't touch the simulation.

### 18.4 Simulation internals
- **ECS-lite** with struct-of-arrays storage per tier.
- **Fixed scheduling order** per tick. Parallel work runs over spatial regions with deterministic per-region
  RNG streams (`seed ⊕ region ⊕ tick`) and ordered merges, so results are thread-count independent.
- **Determinism hygiene:** no `HashMap` iteration in logic (use `BTreeMap`/indexed vecs), no wall-clock, no
  `f32` transcendental functions from the platform libm (use the pure-Rust `libm` crate), no fast-math.
  A CI test runs the same seed on x86-64 and arm64 and compares state hashes.

### 18.5 Persistence and unattended operation
- **Snapshots** every 10 wall-minutes (zstd), keeping the last 12 + one per real day for 7 days.
- **Crash recovery:** restart from the latest snapshot and replay to the logged tick (deterministic), with no gap in the
  event log.
- **Log rotation & caps:** event segments, thought logs and hub logs have per-category disk quotas, and the oldest
  non-archival data is pruned first. Archives (Hall of Fallen Tanks) are kept forever but compacted.
- **Supervisor:** `launchd` keeps each process alive. A watchdog checks liveness endpoints (sim tick
  advancing, hub responsive, renderer FPS heartbeat, OBS websocket stream status) and restarts the failed component.
- **Nightly renderer reload** at ~03:00 Chicago during an aquarium-mode shot (crossfade to black for 2 s).
- **Health page:** small HTTP status page + JSON (uptime, tick rate, memory, FPS, queue depths, last
  errors). Remote access via Tailscale or similar is an open question (OQ-7).

---

## 19. Language and runtime recommendations

### 19.1 Simulation: Rust (D-010)
- Millions of strain-cells, 10⁴–10⁵ organisms and thousands of minds need tight memory layout, no GC pauses
  over weeks of uptime, and multicore determinism. Rust gives roughly 3–10× the headless throughput of
  TS/Bun for this kind of workload, which matters for the batch-testing loop in every phase.
- Hub, renderer, content and assets are **TypeScript** (Node 22 LTS), since the browser and three.js need TS
  anyway. Two languages, cleanly split at a documented protocol.
- Rejected: all-TypeScript (simpler, but slower batches and GC risk at scale), C++ (no benefit over Rust,
  worse safety), Zig (ecosystem).

### 19.2 Runtime LLM
- **Recommendation:** an **~8B instruct model at 4-bit** served by **llama.cpp `llama-server`** (Metal),
  with **JSON-schema-constrained output**, which matters for the `{speaker, text, fact_ids}` grounding format.
  Candidates: Qwen3-8B-class or Llama-3.1-8B-class, or the best open ~8B at build time. Phase 6 benchmarks
  quality, speed and memory.
- MLX-LM is the alternative (often faster on Apple silicon), to be benchmarked side by side in Phase 6.
- Context kept small (4–8k). Prompts are short structured fact sheets. Fallback to templates on timeout or
  grounding failure.

### 19.3 TTS
- **News team:** **Kokoro-82M** (Apache-2.0, small, multiple distinct voices, fast on CPU). Fallback:
  macOS `say` voices.
- **Inhabitants:** **espeak-ng** phoneme input + DSP filters per species (§9.4). It is deliberately
  non-human-sounding and cheap.

---

## 20. Memory, CPU and GPU budget

### 20.1 Memory (16 GB unified), estimates to verify on hardware
| Component | Budget | Notes |
|---|---|---|
| macOS + services | 3.0 GB | |
| OBS (Browser Source + encoder) | 1.2 GB | VideoToolbox H.264 |
| Renderer (Chromium page) | 1.5 GB | nightly reload caps growth |
| tank-sim | 1.5 GB | fields ~60 MB, agents/minds ≤ 600 MB, buffers |
| tank-hub | 0.6 GB | lookahead buffer + SQLite cache |
| LLM (8B Q4 + 8k ctx) | 5.5 GB | **largest risk**; can drop to a 4B model or load on demand |
| TTS (Kokoro) + espeak | 0.6 GB | |
| **Headroom** | **2.1 GB** | |

Sim + renderer ≈ 3.6 GB (≤ 6 GB target ✓). LLM ≈ 5.5 GB (≤ 6 GB target, tight ✓).

### 20.2 CPU
The M4 has 4 performance cores + 6 efficiency cores. Targets: simulation ≤ 2 P-cores average, hub ≤ 0.5, renderer
main thread ≤ 1, OBS and encoder mostly on hardware, TTS on E-cores.

### 20.3 GPU contention (important risk)
LLM generation and three.js both use the GPU. Long generations can **drop render frames**.
Mitigations: (1) generate content in bounded chunks during low-render-load periods (aquarium mode,
wide shots); (2) cap LLM throughput or run partially on CPU; (3) renderer FPS heartbeat → hub pauses
LLM jobs when FPS dips; (4) everything is prepared well ahead of air time. Measured in Phase 6.

### 20.4 I can't measure this from the dev environment
Development runs in a Linux cloud container. **Benchmark and soak scripts** (`tools/bench/`) will ship from
Phase 1 onward. You run them on the Mac mini and the reports go into `docs/reports/`.

---

## 21. Tank lifecycle and endings

Decided in Q5 (D-011): end conditions are fixed in config **before** a run and evaluated by the observer.

| Ending | Condition [tune] |
|---|---|
| **Sterile** | No life after N cycles of chemistry |
| **Extinction** | Zero living organisms |
| **Stagnation** | No new era reached, no discovery, and population variance below ε for X sim-cycles **or** Y real days (e.g. 3 days in ooze-only, 7 days in biological-only with no social species) |
| **Sapient extinction** (not an ending) | The tank continues; the archive notes it |
| **Long life** | No ending. A thriving tank runs indefinitely. Optional `max_real_days` (default off) |

On ending: a final Chronicle entry, a "Funeral of the Tank" broadcast segment (camera tour, archive card), archive
style genomes, signature variants, languages and strata, then seed a new tank (new seed from a
deterministic seed sequence logged in the archive).

Q6 harshness target (D-013), measured across ≥ 200 seeds and tuned via presets:

| Reaches | Target share of tanks |
|---|---|
| Life | ~90% |
| Biological | ~75% |
| Social | ~50% |
| Cultural | ~25% |
| Civilizational | ~10% |
| First Contact (of civilizational) | ~2% |

---

## 22. Risks and mitigations

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | **Civilizations never emerge** (sapience doesn't evolve or doesn't progress) | High | Fatal | Tiered promotion (§5); capacity cost/benefit tuned across hundreds of seeds; Phase 1 proves minds *alone* produce culture before evolution is wired in; harshness presets; measured target shares (§21) |
| R2 | **Boring equilibrium** (stable, nothing happens) | High | High | Midwest climate variability; storyteller tension curve; evolving pathogens; resource depletion feedback; status competition and grudges; stagnation ending as backstop; "change per hour" metric in every batch report |
| R3 | **Performance collapses at scale** | Medium | High | Super-individuals; event-driven minds; capacity-scaled features; mind LOD for unwatched regions (coarser heartbeat, never different rules); hard caps with merge; benchmarks per phase |
| R4 | **Minds all behave the same** | Medium | High | Heritable personality + experience drift; per-agent RNG noise; divergent beliefs/memories; metrics: behavior-entropy and action-distribution divergence across individuals; manual thought-log reading every round |
| R5 | **Instant extinction / runaway** | Medium | Medium | Batch survival curves; config presets; no rescue code (failure is real), but tuning defaults so the *distribution* matches §21 |
| R6 | **Determinism breaks** (cross-platform floats, parallel order) | Medium | High | §18.4 hygiene; CI dual-arch hash test; replay test in every phase |
| R7 | **LLM hallucinates or is ungrounded** | High | Medium | Fact-ID grounding + extraction checks + template fallback; audit archive |
| R8 | **GPU contention drops frames** | Medium | High | §20.3 |
| R9 | **Memory leaks over weeks** | Medium | High | Bounded everything; soak tests with RSS slope reports; nightly renderer reload; process restarts are safe |
| R10 | **Illegible screen** | Medium | High | Expressive-face gate; overlays; spotlight thought subtitles; Director hold times; legibility review with sample clips in Phase 4–5 |
| R11 | **Asset backlog / art never catches up** | Medium | Low | Procedural generator always works; priority queue; pre-launch batch |
| R12 | **Newscasts get repetitive** | High | Medium | Novelty-weighted story selection; template rotation; team memory of past broadcasts; slow news days can feature archives |
| R13 | **Language system too abstract to show viewers** | Medium | Medium | Spoken sound (§9.4), glossed names, "Languages of the Tank" segment, family tree graphic |
| R14 | **Scope** (this is a multi-month project) | Certain | High | Strict phases with exit criteria (ROADMAP); flag growth early; cut list maintained in OPEN_QUESTIONS |
| R15 | **Most stream time is ooze** | Medium | High | Deep-time fast-forward (§2.2); "aquarium beauty" camera covers quiet periods; target ≥ 50% of stream hours with social-tier life (tracked metric) |
| R16 | **Historical climate dataset licensing or availability** | Low | Low | NOAA GHCN-Daily is public domain; bundled at dev time; checksum in config |
| R17 | **Platform policy** (24/7 simulated content with violence, death) | Low | Medium | Stylized, non-graphic depiction; review Twitch/YouTube policies before Phase 5 |

---

## 23. Glossary

- **Agent:** any simulated living thing (possibly a super-individual).
- **Capacity:** the evolved `neural_capacity` gene. It sets how rich a mind is.
- **Culture:** a group sharing a language community + style genome + belief cluster (detected).
- **Descriptor:** the compositional description of a discovery/object, used by rendering and assets.
- **Lookahead:** the buffer by which the simulation runs ahead of the broadcast.
- **Observer:** read-only analysis inside the simulation process (eras, groups, roles, metrics). It never writes world state.
- **Super-individual:** one agent standing for `m` identical-ish organisms.
- **The Lamp:** the tank's light source; it defines days.
