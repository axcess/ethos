# Batch report: seeds 1..24, config 'default', scenario 'baseline'

24 tanks × 60 cycles. Engine 0.1.0.

## Exit criteria (ROADMAP Phase 1)

| Criterion | Target | Result | Pass |
|---|---|---|---|
| Groups form *and* split | ≥ 60% of seeds | 100% | ✅ |
| ≥ 4 distinct roles emerge | ≥ 50% of seeds | 100% | ✅ |
| A culture-specific *wrong* belief appears | ≥ 50% of seeds | 100% | ✅ |
| ≥ 2 mutually unintelligible lexicons | ≥ 30% of seeds | 0% | ❌ |
| Knowledge is lost at least once | ≥ 30% of seeds | 92% | ✅ |

## Outcomes

- Extinct: 29% of tanks
- Any wrong belief held by someone: 100%
- A belief spread from one originator to ≥ 6 people ("prophet"): 100%
- An anti-theft norm took hold in a camp: 46%
- A lost discovery was rediscovered: 75%

## After a boom (population ≥ 1.5× its early level)

| Outcome | Tanks |
|---|---|
| no boom | 1/24 |
| boomed and held (escaped) | 0/24 |
| boomed, drifted down < 30% | 0/24 |
| boomed then crashed ≥ 30% | 18/24 |
| crashed, then regrew past the old peak | 5/24 |

Mean crash depth: 70% of the peak population.

## Metric distributions across seeds

| Metric | mean | median | min | max |
|---|---|---|---|---|
| final population | 105.29 | 112.00 | 0.00 | 275.00 |
| peak population | 208.33 | 209.00 | 111.00 | 275.00 |
| births | 368.58 | 400.00 | 173.00 | 537.00 |
| deaths: starvation | 149.33 | 157.00 | 4.00 | 228.00 |
| deaths: gulpers | 60.75 | 53.00 | 12.00 | 156.00 |
| deaths: fever | 31.12 | 28.00 | 14.00 | 53.00 |
| deaths: old age | 76.62 | 81.00 | 39.00 | 112.00 |
| deaths: cold | 24.17 | 19.00 | 3.00 | 77.00 |
| cooperation rate (requests granted) | 0.82 | 0.82 | 0.75 | 0.90 |
| shares per capita per cycle | 27.38 | 26.50 | 16.93 | 46.09 |
| groups formed | 22.25 | 23.00 | 9.00 | 41.00 |
| group splits | 26.38 | 25.00 | 10.00 | 51.00 |
| camps founded | 28.08 | 27.00 | 12.00 | 56.00 |
| max distinct roles | 9.29 | 9.00 | 7.00 | 12.00 |
| mean belief diversity (Jaccard) | 0.75 | 0.76 | 0.66 | 0.82 |
| culture-specific wrong beliefs (max) | 6.33 | 6.00 | 3.00 | 14.00 |
| inventions | 31.50 | 33.00 | 13.00 | 50.00 |
| distinct discoveries | 15.54 | 16.00 | 7.00 | 23.00 |
| discoveries per generation | 11.49 | 11.33 | 5.67 | 18.82 |
| knowledge lost | 5.96 | 4.00 | 0.00 | 18.00 |
| languages (max) | 1.00 | 1.00 | 1.00 | 1.00 |
| min intelligibility | 0.88 | 0.89 | 0.78 | 0.97 |
| final lexicon coherence | 0.75 | 0.91 | 0.00 | 0.98 |
| grammar tier (proxy, max) | 3.00 | 3.00 | 3.00 | 3.00 |
| behavior entropy (per person) | 0.45 | 0.44 | 0.41 | 0.49 |
| behavior divergence (between people) | 0.05 | 0.05 | 0.03 | 0.06 |
| change index (mean) | 0.26 | 0.26 | 0.16 | 0.32 |
| change index (min, 2nd half) | 0.16 | 0.17 | 0.00 | 0.21 |
| thefts | 66.33 | 67.00 | 23.00 | 125.00 |
| punishments | 28.08 | 17.00 | 0.00 | 103.00 |
| decisions / sec | 91492.55 | 91047.10 | 80619.34 | 103869.62 |
| wall seconds | 35.94 | 39.23 | 15.24 | 57.64 |

## Survival curve

| cycle | tanks alive | median pop |
|---|---|---|
| 0 | 24/24 | 80 |
| 6 | 24/24 | 107 |
| 12 | 24/24 | 127 |
| 18 | 24/24 | 144 |
| 24 | 24/24 | 160 |
| 30 | 23/24 | 134 |
| 36 | 22/24 | 103 |
| 42 | 21/24 | 87 |
| 48 | 19/24 | 81 |
| 54 | 17/24 | 93 |
| 59 | 17/24 | 111 |

## Per-seed

| seed | pop | peak | after boom | splits | roles | languages | inventions | lost | culture-false | extinct |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 275 | 275 | crash_recovered | 25 | 10 | 1 | 42 | 2 | 7 | false |
| 2 | 116 | 205 | crash | 28 | 9 | 1 | 31 | 3 | 5 | false |
| 3 | 95 | 256 | crash | 41 | 8 | 1 | 32 | 6 | 7 | false |
| 4 | 166 | 211 | crash | 38 | 9 | 1 | 34 | 0 | 6 | false |
| 5 | 183 | 235 | crash | 51 | 9 | 1 | 42 | 2 | 8 | false |
| 6 | 0 | 183 | crash | 19 | 9 | 1 | 24 | 12 | 3 | true |
| 7 | 162 | 205 | crash | 33 | 8 | 1 | 24 | 4 | 7 | false |
| 8 | 0 | 211 | crash | 13 | 10 | 1 | 32 | 15 | 4 | true |
| 9 | 220 | 220 | crash_recovered | 21 | 11 | 1 | 41 | 1 | 9 | false |
| 10 | 188 | 187 | crash_recovered | 15 | 9 | 1 | 39 | 10 | 6 | false |
| 11 | 44 | 213 | crash | 28 | 8 | 1 | 34 | 2 | 7 | false |
| 12 | 188 | 205 | crash_recovered | 43 | 10 | 1 | 33 | 4 | 9 | false |
| 13 | 105 | 246 | crash | 27 | 9 | 1 | 50 | 6 | 4 | false |
| 14 | 90 | 209 | crash | 31 | 12 | 1 | 35 | 6 | 6 | false |
| 15 | 72 | 158 | crash | 21 | 9 | 1 | 27 | 0 | 5 | false |
| 16 | 113 | 186 | crash | 45 | 10 | 1 | 34 | 4 | 9 | false |
| 17 | 0 | 204 | crash | 19 | 10 | 1 | 34 | 18 | 5 | true |
| 18 | 0 | 205 | crash | 19 | 9 | 1 | 19 | 13 | 6 | true |
| 19 | 0 | 234 | crash | 19 | 9 | 1 | 25 | 12 | 4 | true |
| 20 | 215 | 215 | crash_recovered | 20 | 9 | 1 | 32 | 1 | 5 | false |
| 21 | 0 | 207 | crash | 12 | 7 | 1 | 13 | 6 | 14 | true |
| 22 | 183 | 249 | crash | 40 | 11 | 1 | 42 | 3 | 4 | false |
| 23 | 0 | 170 | crash | 15 | 8 | 1 | 20 | 11 | 4 | true |
| 24 | 112 | 111 | none | 10 | 10 | 1 | 17 | 2 | 8 | false |
