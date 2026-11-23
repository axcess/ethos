# Phase 1 Audit and Completion Goal

**Date:** 2026-11-23
**Goal:** Phase 1 is complete only when every box in the "Definition of done" below is checked. Phase 2 does not start before then.

Status key: ✅ done · 🟡 partial · ❌ missing

---

## 1. Scope (ROADMAP Phase 1) vs. what exists

| Scope item | Status | Notes |
|---|---|---|
| Simplified world: terrain, light/temperature/food, deterministic RNG, snapshots, event log | ✅ | 2-D grid (D-024); event log is JSONL rather than binary segments, which is fine for Phase 1 |
| Chicago climate driver with bundled dataset | ✅ | NOAA 1929–2025, checksum recorded |
| Mind module at full sapient capacity | ✅ | Drives, personality, emotions, memory, beliefs, prediction, theory of mind, utility + intents |
| **Mind capacity scaled by the `capacity` gene (D-005)** | ❌ | **The gene only affects invention odds.** Memory, belief, acquaintance and vocabulary limits are fixed config values. Phase 2 depends on this: animals and sapients must run the same mind at different capacities. |
| Cooperation: requests, sharing, joint tasks, remembered follow-through | ✅ | Hunts with fair or unfair splits, deserters remembered |
| Groups: detected by the observer | ✅ | Label propagation, with splits, merges and dissolves |
| **Groups: believed by members ("we are the X")** | ❌ | Camps act as believed units, but no mind holds a group identity or name; `Kind::Us`/`Kind::Them` are unused. In-group trust and out-group wariness are only indirect (via culture of origin). |
| Roles: detected + believed | ✅ | 14 observer roles; minds hold Heals/Leads/Teaches/Thief beliefs from observation |
| Language: naming game, phonology, sound change | ✅ | Plus lexical replacement and child mishearing |
| Life records, notability, thought log | ✅ | |
| Discovery Registry + ~25-node subgraph | ✅ | 26 nodes, material variants, loss and rediscovery |
| Genome type defined in full; seeded distribution | ✅ | |
| CLI: run, batch, inspect | ✅ | Also scenarios, determinism, bench |
| CLI: `tank thoughts --agent ID` | 🟡 | Covered by `inspect` (last 16 thoughts) and `thoughts.txt`; no dedicated command |
| CLI: `tank replay --to TICK` | ❌ | Snapshot/restore exists and is tested, but there is no "replay to a given moment" command |
| Determinism test (same seed → same hash; snapshot restore) | 🟡 | ✅ on Linux; **unverified on macOS and Windows** |
| `tools/bench/` benchmark for the Mac mini | 🟡 | Script exists; **no Mac results yet** |

## 2. Scenario tests

All eight exist (famine, predator surge, stranger, leader death, windfall, plague, harsh winter, drought year).

| Requirement | Status | Notes |
|---|---|---|
| Each scenario produces a qualitatively different response | 🟡 | Famine, predator surge, stranger, leader death and plague are clear. Harsh winter and drought are within noise on 6 seeds. Results predate the boom-bust and food-bug changes, so they must be re-run. |

## 3. Exit criteria (latest batch: 24 seeds × 60 cycles, `phase-1-batch-boombust.md`)

| Criterion | Target | Latest | Status |
|---|---|---|---|
| 200-mind × 20-cycle tank < 5 min on one core | — | ~5–15 s | ✅ |
| Groups form and split | ≥ 60% | 100% | ✅ |
| ≥ 4 distinct roles | ≥ 50% | 100% | ✅ |
| A culture-specific wrong belief | ≥ 50% | 100% | ✅ |
| ≥ 2 mutually unintelligible lexicons | ≥ 30% | **0%** | ❌ |
| Knowledge lost at least once | ≥ 30% | 96% | ✅ |
| Sampled thought logs judged believable and varied | written assessment | Done before leadership, norms and boom-bust | 🟡 needs re-review |
| Scenario responses differ from baseline | — | See §2 | 🟡 |

## 4. Problems found since the exit criteria were set

| Problem | Severity | Notes |
|---|---|---|
| Boom-bust: nearly every tank overshoots and crashes; none escape | High | OQ-17. Recommended fix: learned restraint |
| Extinction 29% by cycle 60 | High | Above the harshness target; likely tied to the boom-bust problem |
| Earlier reports contain bug-affected numbers | Medium | `phase-1-batch-24x60.md` and `phase-1-scenarios.md` predate the shellfish food bug fix |
| Shortcuts that aren't real decisions | Medium | Infant/child feeding runs automatically; fixed matrilocal residence; one random member decides camp membership |
| Behavior divergence between individuals is low (0.03–0.04) | Low | Most people forage most of the time; specialization may need Phase 3 economics |
| Grammar tier is a proxy (vocabulary size and agreement), not syntax | Low | Acceptable for Phase 1; documented |
| Only 6 automated tests; none for mind components | Medium | Regressions (like the food bug) are found by accident |
| No food-balance check | Medium | The shellfish bug ran for days because nothing asserted that food gathered ≤ food grown |
| `gentle` / `harsh` presets never batch-tested | Low | |
| Windows build unverified on a real PC | Low | |

---

## Definition of done (Phase 1 completion goal)

**Must have (blocks Phase 2):**

- [ ] **D1. Capacity-scaled minds (D-005).** Memory, belief, acquaintance, vocabulary and planning limits scale with the `capacity` gene. A test shows a low-capacity mind runs the same code with fewer features.
- [ ] **D2. Group identity.** Minds can hold "I belong to group X" with a shared name in their language, giving in-group trust and out-group wariness. The observer can report believed vs. detected groups.
- [ ] **D3. Boom-bust has real escapes.** Learned restraint (OQ-17): across 24 seeds × 60 cycles, at least 20% of booming tanks escape *and* at least 30% crash.
- [ ] **D4. Extinction by cycle 60 ≤ 15%** on the default preset.
- [ ] **D5. Languages.** Either ≥ 2 mutually unintelligible lexicons in ≥ 30% of seeds by cycle 200 (on the `longrun-test` preset), or you explicitly accept "dialects only" for Phase 1 and the criterion moves to Phase 3.
- [ ] **D6. Food-balance safeguard.** An automated test asserts that, over a run, food gathered never exceeds regrowth plus starting stock (catches any repeat of the shellfish bug).
- [ ] **D7. Re-run and replace stale reports.** A fresh 24-seed batch, the scenario suite (≥ 12 seeds, so harsh winter and drought can be judged), and preset batches for `gentle` and `harsh`.
- [ ] **D8. Thought-log re-review.** A written assessment with excerpts covering leadership, norms, boom-bust collapse and recovery.
- [ ] **D9. Cross-platform determinism.** Mac and Windows produce the reference hash (needs your two runs).
- [ ] **D10. Mac benchmark results** committed to `docs/reports/`.

**Should have (do if cheap; otherwise move to Phase 2 with a note):**

- [ ] S1. Child feeding as a real parental decision (not automatic).
- [ ] S2. Residence after bonding decided by the couple, not a fixed rule.
- [ ] S3. Camp membership decided by more than one member.
- [ ] S4. `tank replay --to TICK` and `tank thoughts --agent ID` commands.
- [ ] S5. Unit tests for contingency learning, belief fading and the naming game.

**Your sign-off:** watch at least two tanks in the viewer and agree the behavior reads as believable.

### Order of work
1. D6 (food-balance test): guards everything after it.
2. D1 (capacity scaling), then D2 (group identity): structural, change behavior.
3. D3/D4 (learned restraint): tuned after D1–D2, since both affect outcomes.
4. D5 long runs, then D7 batches, D8 review.
5. D9/D10 on your hardware in parallel, whenever convenient.
