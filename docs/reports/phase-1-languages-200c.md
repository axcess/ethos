# Batch report: seeds 1..8, config 'longrun-test', scenario 'baseline'

8 tanks × 200 cycles. Engine 0.1.0.

## Exit criteria (ROADMAP Phase 1)

| Criterion | Target | Result | Pass |
|---|---|---|---|
| Groups form *and* split | ≥ 60% of seeds | 100% | ✅ |
| ≥ 4 distinct roles emerge | ≥ 50% of seeds | 100% | ✅ |
| A culture-specific *wrong* belief appears | ≥ 50% of seeds | 100% | ✅ |
| ≥ 2 mutually unintelligible lexicons | ≥ 30% of seeds | 100% | ✅ |
| Knowledge is lost at least once | ≥ 30% of seeds | 100% | ✅ |

## Outcomes

- Extinct: 0% of tanks
- Any wrong belief held by someone: 100%
- A belief spread from one originator to ≥ 6 people ("prophet"): 100%
- An anti-theft norm took hold in a camp: 12%
- A lost discovery was rediscovered: 100%

## After a boom (population ≥ 1.5× its early level)

| Outcome | Tanks |
|---|---|
| no boom | 0/8 |
| boomed and held (escaped) | 0/8 |
| boomed, drifted down < 30% | 3/8 |
| boomed then crashed ≥ 30% | 1/8 |
| crashed, then regrew past the old peak | 4/8 |

Mean crash depth: 35% of the peak population.

## Metric distributions across seeds

| Metric | mean | median | min | max |
|---|---|---|---|---|
| final population | 251.25 | 248.00 | 226.00 | 298.00 |
| peak population | 280.12 | 284.00 | 251.00 | 311.00 |
| births | 2212.62 | 2251.00 | 1943.00 | 2605.00 |
| deaths: starvation | 1083.88 | 1103.00 | 888.00 | 1285.00 |
| deaths: gulpers | 13.88 | 10.00 | 4.00 | 46.00 |
| deaths: fever | 244.62 | 259.00 | 156.00 | 392.00 |
| deaths: old age | 570.62 | 582.00 | 469.00 | 784.00 |
| deaths: cold | 128.38 | 115.00 | 93.00 | 187.00 |
| cooperation rate (requests granted) | 0.95 | 0.95 | 0.94 | 0.96 |
| shares per capita per cycle | 43.55 | 42.96 | 40.87 | 48.71 |
| groups formed | 225.75 | 237.00 | 146.00 | 294.00 |
| group splits | 187.62 | 174.00 | 136.00 | 255.00 |
| camps founded | 24.50 | 25.00 | 12.00 | 48.00 |
| believed communities (max) | 11.12 | 11.00 | 9.00 | 13.00 |
| detected groups agreeing on one identity | 0.34 | 0.34 | 0.29 | 0.38 |
| max distinct roles | 9.75 | 10.00 | 9.00 | 11.00 |
| mean belief diversity (Jaccard) | 0.82 | 0.82 | 0.79 | 0.84 |
| culture-specific wrong beliefs (max) | 3.38 | 3.00 | 2.00 | 5.00 |
| inventions | 74.25 | 76.00 | 49.00 | 106.00 |
| distinct discoveries | 24.12 | 25.00 | 21.00 | 26.00 |
| discoveries per generation | 7.43 | 7.60 | 4.90 | 10.60 |
| knowledge lost | 12.12 | 11.00 | 5.00 | 19.00 |
| languages (max) | 4.75 | 5.00 | 3.00 | 7.00 |
| min intelligibility | 0.21 | 0.20 | 0.14 | 0.31 |
| final lexicon coherence | 0.75 | 0.76 | 0.71 | 0.78 |
| grammar tier (proxy, max) | 3.00 | 3.00 | 3.00 | 3.00 |
| behavior entropy (per person) | 0.51 | 0.51 | 0.50 | 0.52 |
| behavior divergence (between people) | 0.05 | 0.05 | 0.04 | 0.05 |
| change index (mean) | 0.29 | 0.29 | 0.26 | 0.31 |
| change index (min, 2nd half) | 0.19 | 0.19 | 0.16 | 0.22 |
| thefts | 44.75 | 43.00 | 11.00 | 135.00 |
| punishments | 11.62 | 5.00 | 0.00 | 66.00 |
| decisions / sec | 97810.76 | 99304.27 | 93761.86 | 102318.20 |
| wall seconds | 179.97 | 175.77 | 152.13 | 214.20 |

## Survival curve

| cycle | tanks alive | median pop |
|---|---|---|
| 0 | 8/8 | 82 |
| 20 | 8/8 | 146 |
| 40 | 8/8 | 169 |
| 60 | 8/8 | 201 |
| 80 | 8/8 | 217 |
| 100 | 8/8 | 245 |
| 120 | 8/8 | 239 |
| 140 | 8/8 | 224 |
| 160 | 8/8 | 237 |
| 180 | 8/8 | 268 |
| 199 | 8/8 | 248 |

## Per-seed

| seed | pop | peak | after boom | splits | roles | languages | inventions | lost | culture-false | extinct |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 269 | 310 | sagged | 213 | 11 | 5 | 93 | 12 | 3 | false |
| 2 | 298 | 311 | crash_recovered | 241 | 10 | 5 | 106 | 11 | 3 | false |
| 3 | 250 | 284 | crash_recovered | 171 | 9 | 6 | 68 | 5 | 5 | false |
| 4 | 226 | 257 | sagged | 255 | 11 | 7 | 59 | 19 | 2 | false |
| 5 | 231 | 297 | sagged | 166 | 10 | 5 | 76 | 10 | 5 | false |
| 6 | 245 | 263 | crash_recovered | 145 | 9 | 3 | 55 | 11 | 2 | false |
| 7 | 243 | 268 | crash | 136 | 9 | 3 | 49 | 11 | 5 | false |
| 8 | 248 | 251 | crash_recovered | 174 | 9 | 4 | 88 | 18 | 2 | false |
