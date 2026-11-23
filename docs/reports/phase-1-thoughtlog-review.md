# Phase 1 Thought-Log Review (D8)

**Date:** 2026-11-23 · **Build:** after D1–D6 and the review fixes · **Main sample:** seed 8, 60 cycles (`tank run --seed 8 --cycles 60`), plus seeds 2, 3, 6 and 9 during the D2–D5 work.

The question: do individual minds read as believable, varied and motivated, and does anything look scripted or degenerate?

## What reads well

**Anticipation (acting before the outcome).** People stockpile and build on the strength of their own learned rules:
> `Bafmtha (21): saw food-low + expects hunger (after light-lengthening, 100%) → forage at the east front shallows because expects-hunger 1.0, stockpiling 1.0`
> `Qngaugthqaub (30, builder/inventor/leader): expects cold-suffering (after ice-overhead, 92%) → build a sand burrow because expects-cold 0.9`

**Leaders provide, even when hungry themselves** (prestige-driven redistribution, D-039):
> `Qngaugthqaub (30, builder/inventor/leader): hunger(0.6) + saw hungry Gmaihmmi → give food to Gmaihmmi because trusts 1.0`

**Desperation and theft are motivated, not random:**
> `Qaighthisfq (21): expects a-death (after early-season, 77%) → steal food from Mabsu because desperate 0.7`

**Grief, teaching and kin care:**
> `Beqqi (30): grieving(1.0) → mourn because grief 1.0`
> `Hfa (25, guard): → teach Mims the burial because kin 0.8, kin-needs-care 0.3`

**Fear interrupts** (`!!` marks an interrupt):
> `Mabsu (2): !! saw gulper → flee because safety 0.5, fear 0.3`

**Lives have arcs.** Notable people found camps, lead, forget skills and die. One leader's life:
> bonded with Thas → gave birth to Nghi → became known as a leader → forgot the bitter-sponge remedy → founded the camp of Haise → invented the shell scraper → died of old age

**The chronicle reads as history:** four founding bands become named peoples in cycles 1–2 ("the Maib-folk"); a breakaway camp gets its own name years later; kelp cordage, burial and the fever remedy are forgotten by cycle 17; cordage is rediscovered at cycle 15 in another camp; "the Lamp governs the seasons" spreads as a belief.

## Found and fixed during this review
| Problem seen in the logs | Fix |
|---|---|
| "expects *plenty*" shown beside "because expects-*hunger*": the log displayed the most probable expectation, not the one driving the choice | Threats now outweigh a happy forecast in what's displayed; hunger expectations now outnumber "plenty" about 5 to 1 |
| One leader founded 3 camps in 21 days | People settle for 2 cycles after founding or joining a camp; nobody now founds more than 2 camps |
| The same belief announced 4 times (several people reached it independently) | One announcement per belief |
| (Earlier in D3/D4) one person beat the same victim 15 times; 20 pairs in mutual vendettas | Grudges license only immediate retaliation; punishment settles the debt |

## Remaining issues (not blocking)
1. **Curiosity often shows as the top drive (1.0).** It builds faster than exploring and tinkering satisfy it. It's cosmetic in the logs but inflates experimenting; tune in Phase 2 when animals share the drive system.
2. **Early superstitions are confident.** In the first weeks people hold rules like "light-lengthening → cold (80%)" from only a handful of days. This is intended (beliefs can be wrong), but the 100% displays look odd; show confidence scaled by evidence.
3. **Birth restraint is invisible in the logs.** Foresight lowers conception odds (D-043), but conception isn't a logged decision, so viewers can't see "we waited to have a child because hard times are coming". Worth a thought record in Phase 2.
4. **Children speak like adults** in logs (e.g. a 2-year-old "flees because safety 0.5"). Accurate but flat; child-specific phrasing belongs with the LLM narration layer (Phase 6).

## Verdict
Behaviour is motivated, varied between individuals, and traceable to perceptions, feelings, expectations and relationships. Nothing observed is scripted. The degenerate patterns found in earlier reviews (food ping-pong, saturated curses, vendettas, all-night socialising, starving infants) are gone, and the three new ones found here are fixed. **D8 passes**, with the four non-blocking issues above carried into Phase 2.
