# Phase 0 Report: Design Document

**Date:** 2026-11-21 · **Status:** awaiting your review

## What was done
- Wrote [`DESIGN.md`](../DESIGN.md) covering every Phase 0 item in the prompt (P§14): systems list, how
  each era emerges, data models (genomes, minds, memories, beliefs, cultures, style genomes, discoveries,
  assets, events), interestingness model, language and runtime recommendations, memory budget and risks.
- Logged 23 decisions in [`DECISIONS.md`](../DECISIONS.md), with alternatives rejected.
- Wrote [`ROADMAP.md`](../ROADMAP.md) with measurable exit criteria for every phase.
- Opened [`OPEN_QUESTIONS.md`](../OPEN_QUESTIONS.md).
- Applied all ten recommendations from the first round of questions, plus the three accepted pitches
  (spoken languages, fossils/strata, tank weather) and your Midwest-weather request.

No code yet, as instructed.

## Most important design choices to review
1. **Tiered simulation with promotion by evolved traits** (§5): how chemistry-to-civilization becomes feasible
   without scripting.
2. **One mind, many capacities** (§5.4, §7): a clever fish and a prophet run the same code.
3. **Midwest climate** (§4, D-017): historical Chicago years drive the tank; live weather is on-screen only.
4. **Lookahead buffer** (§2.3, D-012): the simulation runs 1–3 min ahead so the camera can anticipate.
5. **Plague as an organism** and a **storyteller that can't see who lives where** (D-019, D-020).

## Things I'm least sure about
- Whether sapience will evolve often enough at the target pacing (risk R1). Phase 2 is where we find out.
  Phase 1 is deliberately designed to prove minds can make culture *before* we depend on evolution producing them.
- The LLM + renderer GPU contention on the M4 (R8). It can't be measured until Phase 6 on your hardware.
- The 16 GB budget is tight with an 8B model. A 4B model is the fallback.

## Questions and ideas for next phase
**Questions (with recommendations)**
1. **OQ-1, Midwest weather mode:** confirm historical-driver + live-flavor? *(Recommended.)*
2. **OQ-3, lookahead delay:** OK with a 1–3 min sim-to-screen gap? *(Recommended: yes.)*
3. **OQ-4, hardware:** macOS version, and can you run benchmark scripts in Phase 1?
4. **Phase 1 discovery subgraph:** OK to include ~25 discovery nodes in Phase 1 (the prompt puts the full
   graph in Phase 3) so we can test need-driven invention and knowledge loss early? *(Recommended: yes, small.)*

**Ideas for you to accept or reject**
- **"Tank Almanac" data file:** each tank exports a compact JSON almanac (eras, populations, languages,
  discoveries, Midwest years). It's useful for news grounding and later for comparing tanks. Small cost.
- **Seed showcase list:** keep a curated list of test seeds that produce particularly interesting tanks
  during development. Pure dev tooling, never used to pick broadcast seeds (that would be authorship).
- **Phase 1 visual debug page:** a minimal 2-D top-down HTML viewer of a headless run (dots, groups,
  speech bubbles from thought logs). Not the renderer, just a debugging aid that makes reviewing Phase 1 far
  easier for both of us. About 1 day of work.
