# Phase 1 Final Status

**Date:** 2026-11-23 · Supersedes `phase-1-report.md` (kept for history).
**Checklist:** `phase-1-audit.md`. **8 of 10 must-haves done; 2 blocked on your hardware.**

| # | Must-have | Status | Evidence |
|---|---|---|---|
| D1 | Capacity-scaled minds | ✅ | D-041, `sim/tests/capacity.rs` |
| D2 | Group identity | ✅ | D-042; named peoples, believed vs detected groups |
| D3 | Boom-bust has real escapes | ✅ | 46% of booms escape, 46% crash (`phase-1-final-batch-24x60.md`) |
| D4 | Extinction ≤ 15% by cycle 60 | ✅ | 0% |
| D5 | Languages | ✅ | Every tank splits into 3–7 languages by cycle 200 (`phase-1-languages-200c.md`) |
| D6 | Food-balance safeguard | ✅ | `sim/tests/food_balance.rs` catches the old bug |
| D7 | Fresh reports | ✅ | Files below |
| D8 | Thought-log review | ✅ | `phase-1-thoughtlog-review.md` |
| D9 | Cross-platform determinism | ⏸ | Needs your Windows PC (`Tank.bat` option 5) and Mac |
| D10 | Mac benchmark | ⏸ | Needs the Mac mini |

## Final numbers (this build)

**24 seeds × 60 cycles, default preset** (`phase-1-final-batch-24x60.md`): groups form and split 100%; ≥ 4 roles 100%; a culture-specific wrong belief 100%; knowledge lost 96%; 0% extinct; after the boom: 11 escape, 2 sag, 10 crash, 1 crashes and regrows; anti-theft norms take hold in 25%. (The language row shows 0% because this batch stops at cycle 60; languages split around cycle 136, see D5.)

**Scenarios, 12 seeds × 40 cycles, injected at cycle 10** (`phase-1-final-scenarios-12seeds.md`), change in final population vs baseline:

| Scenario | Δ pop | Notes |
|---|---|---|
| Predator surge | −82 | +66 gulper deaths; thefts more than double |
| Leader death | −19 | more starvation; order frays |
| Plague | −19 | few direct fever deaths; the disruption shows up as starvation |
| Windfall | −19 | the bloom feeds drifters, then gulpers (emergent) |
| Drought year | −7 | weak: an aquatic tank buffers drought |
| Stranger | −5 | more starvation |
| Harsh winter | −5 | +12 starvation |
| Famine | −3 | weak at cycle 40: births drop more than deaths rise, so populations rebound |

**Presets, 12 seeds × 60 cycles:** gentle: 9/12 booms escape, 0% extinct. Harsh: 8/12 booms crash, 0% extinct, population about 35% lower than gentle.

## Known weaknesses carried into Phase 2
1. Famine, drought and harsh winters are mild in the long run; famine's main effect is fewer births.
2. "Harsh" never drives a tank extinct within 60 cycles; the preset needs a harder edge if failure is to be common.
3. Thought-log issues: curiosity saturates, early superstitions display as 100%, birth restraint isn't visible, children speak like adults.
4. Behaviour divergence between individuals stays low; specialisation likely needs Phase 3 economics.
5. Gulpers matter much less now that they follow prey: about 7 deaths per tank, down from 50–70.
