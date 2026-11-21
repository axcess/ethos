# Phase 1 Report: Headless Minds Engine

**Date:** 2026-11-21 · **Status:** stopped at a checkpoint for your review (not yet complete; see "Not done")

## What exists
- **`sim/`: Rust engine, deterministic.** Same seed gives the identical state hash, and a snapshot restored mid-run continues to the identical hash (`cargo test`, `tank determinism`).
- **Minds (DESIGN §7):** drives, heritable personality with drift, appraisal emotions, salience-ranked episodic memory, beliefs that can be wrong and that fade, contingency-learned predictions ("light-shortening → hunger"), theory of mind (trust, affinity, respect, debts, grudges, observed roles), and utility decisions with anticipatory intents (stockpiling before expected scarcity).
- **Cooperation (§8):** asking for food, sharing, joint drifter hunts with fair or unfair splits, deserters remembered, camp splits and new camps, pair bonds, infant care by kin.
- **Conflict and law:** theft, punishment, restitution law, anti-theft norms spreading from witnessed punishments.
- **Language (§9):** per-culture sound systems, a naming game, regular sound change spreading word by word, synonym replacement, children's mishearings, names that mean something.
- **Discovery (§10):** a 26-node aquatic subgraph; invention is need-driven; teaching, imitation and tally-mark records; skills fade unless practised, so knowledge can be lost and rediscovered; Discovery Registry with material variants.
- **Climate:** real Chicago NOAA daily data, 1929–2025 (one tank cycle = one real year).
- **Observer (read-only):** groups, 14 roles, languages, belief diversity, false beliefs, notability, change index.
- **CLI:** `run`, `batch`, `scenarios`, `determinism`, `bench`, `inspect`. Presets: `config/presets/{gentle,default,harsh}.toml`.
- **Debug viewer:** `tools/viewer/index.html` (approved extra).

## Results (24 seeds × 60 cycles, final build): `phase-1-batch-24x60.md`
| Exit criterion | Target | Result |
|---|---|---|
| Groups form and split | ≥ 60% | **100%** ✅ |
| ≥ 4 distinct roles | ≥ 50% | **100%** ✅ (median 8) |
| A culture-specific wrong belief | ≥ 50% | **83%** ✅ |
| ≥ 2 mutually unintelligible lexicons | ≥ 30% | **0%** ❌ (dialects only; see below) |
| Knowledge lost at least once | ≥ 30% | **96%** ✅ (96% later rediscover something) |
| 200-mind × 20-cycle run < 5 min on one core | — | ✅ about 5–15 s (65k–206k decisions/s, 20–60 MB RSS) |

Other results: 0% extinction by cycle 60. Requests are granted 86% of the time, and there are about 47 thefts per tank. Causes of death are, in order, starvation, gulpers, old age, cold and fever.

**Scenarios** (`phase-1-scenarios.md`, 6 seeds): famine cuts population by 71; predator surge adds 40 gulper deaths. A stranger raises thefts and punishments. Windfall *lowers* population, because the bloom feeds the herds, the herds feed the gulpers, and the gulpers eat people. That ecological side effect emerged on its own. Losing the leader has almost no effect. After recalibration, harsh winters now kill through cold.

## What reading the thought logs found (and fixed)
Most tuning came from reading individual minds rather than averages. Examples:
- Infants starving while their parents gave food away.
- Superstitions ("kelp is cursed") rising to 100% in everyone, which silently stopped invention.
- Two starving relatives passing the same food back and forth until both died.
- People calling their own mate a "stranger".
- All-night socializing.
- A mating drive at 38.
- A camp with six different words for "food".

Each fix is logged in DECISIONS D-024–D-036. Current logs read as motivated and varied, for example:
`[c3d11] Ivkiit (17, inventor/thief): grieving(0.5) + hunger(1.0) + expects gulper-attack (after early-season, 50%) → forage at the west front kelp`

The most notable lives are often heroic guards who drive off several gulpers and are eventually taken by one, and outcast prophets whose curses spread.

## Honest weak spots
1. **Languages:** within 60 cycles (about 2 generations), groups form dialects. Cross-group intelligibility gets as low as 0.49, while agreement within groups stays at 0.90. Separate languages haven't appeared. The 200-cycle test was cut short. Early "many languages" readings were measurement artifacts, now fixed (D-028).
2. **Early crash:** the median population falls from 126 to about 52 by cycle 6, then recovers slowly. The founders' first winters are brutal.
3. **Leaders matter little,** and behaviour divergence between people is low (0.04): most people forage most of the time.
4. **Agriculture-like growth:** algae gardens let some tanks reach about 800 people by cycle 60.
5. **Anti-theft norms take hold in only 12% of tanks;** punishment is rare after the feud fix.
6. The grammar tier is a proxy based on vocabulary size and agreement; there is no real syntax yet.

## Not done
- The 200-cycle language runs (stopped at your request).
- A clean `tank bench` table on an idle machine.
- Mac numbers: run `bash tools/bench/run_mac_bench.sh` and send the file.
- The determinism check was not re-run on the very last commit (the `cargo test` determinism tests pass).

## Questions and ideas for next round
1. **Language timescale:** should real languages split within about 2 generations (faster drift), or is "dialects by cycle 60, languages later" acceptable? *Recommend:* run 200-cycle tests first, then decide.
2. **Early crash:** soften the founders' first winters (start with more stored food), or keep the bottleneck as drama? *Recommend:* soften it modestly.
3. **Agriculture:** make the algae garden harder to reach (calendar prerequisite, larger group) now, or leave it for Phase 3? *Recommend:* add the prerequisite now; it's cheap.
4. **Leadership:** give leaders real effects (coordinating hunts, settling disputes, choosing where to move) so that losing one matters? *Recommend:* yes, before Phase 2.
