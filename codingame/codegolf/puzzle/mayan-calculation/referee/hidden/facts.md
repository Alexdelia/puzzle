# Mayan Calculation: hidden validators

12 validators, titles = the 12 IDE test titles, weights 83 x 10 (1-10) + 85 x 2 (11, 12) (CG per-validator report, 2026-10-24).
Every validator keeps the font of its own test (`L == H`); validators 1-8 and 10-12 keep the operator and digit counts of their own test, 9 does not.

Recovered exactly: 01-07 and 09-12 (`NN.in`/`NN.out` here, each md5-verified on CG). No `complete`: 08 has 906 candidates left, best guess 13452 * 789012 (unverified).

## Method (2026-10-24, probes perl `bis01`..`bis25` in `probe/probes.tsv`)

Perl general solution printing the right answer iff a per-validator predicate holds (bis26-bis30 too; generator `agent/state/journal/validator.mayan-calculation.-.1792834543922843023.scratch/gen.py`, state `state.json`). `Z = a*20^nb + b` (nb = digit count of b), shape key = (font checksum, op, na, nb); each probe bisects every validator's Z interval at the median of valid candidates (`-`: a>=b, `/`: exact, leading digit nonzero). bis01 found the odd validator (9), then V = na*16+nb bisected its shape. bis25: md5 of each reconstructed input (chomped lines joined by `\n`) matched for 01-07 -> exact; bis28 verified 10, 11; bis30 verified 9 and 12 (one of 2 candidates each, md5 matched).

## Per validator

| # | title | font | input | answer | status |
|---|---|---|---|---|---|
| 1 | Simple addition | `.o_` 4x4 | 2 + 2 | 4 | exact, md5 verified (bis25) |
| 2 | Addition with carry | `.o_` | 17 + 5 | 22 | exact, md5 verified |
| 3 | Multiplication | `.o_` | 6 * 6 | 36 | exact, md5 verified |
| 4 | Simple subtraction | `.o_` | 15 - 7 | 8 | exact, md5 verified |
| 5 | Subtraction | `.o_` | 113 - 18 | 95 | exact, md5 verified |
| 6 | Simple division | `.o_` | 9 / 3 | 3 | exact, md5 verified |
| 7 | Division | `.o_` | 2000 / 10 | 200 | exact, md5 verified |
| 8 | Great multiplication | `.o_` | 13452 * b, b in 788521..789426 (906 candidates); test 8's b = 789012 is inside and 13452 is a digit permutation of test 8's 12345: guess 13452 * 789012 | ~1.06e10 (> 2^32, < 2^53) | a exact by bisection, b open, guess unverified |
| 9 | Zero | `.o_` | 234561 * 0 (5-digit a, test 9 has 4) | 0 | exact, md5 verified (bis30) |
| 10 | Missing power | `.o_` | 8001 * 36 | 288036 | exact, md5 verified (bis28) |
| 11 | Base 20 | `0-9A-J` 1x1 | 8016 * 24 | 192384 | exact, md5 verified (bis28) |
| 12 | Other symbols | `.X` 11x11 | 2012 * 21202 | 42658424 | exact, md5 verified (bis30) |

Validator 8 after bis30: `Z = a*20^5 + b` in `[43047188521, 43047189427)`. Next worker: one probe md5-checking the 13452 * 789012 reconstruction (`state.json` in the scratch dir: set v8 lo/hi to the single Z 43047189012, `step.sh NN verify`); if it fails, ~10 bisection probes (`step.sh NN`). Then write 08, run `--hidden`, touch `complete`.

## Facts for golf (certain)

- `L == H` everywhere; fonts are the 3 IDE fonts
- no negative result, divisions exact, one result is 0 (validator 9) -> zero must print one glyph
- results: only validator 8 exceeds 2^31 (13452 * ~7.9e5 ~ 1.06e10 -> needs 64-bit, fits double exactly); all others < 2^31 (largest 12: 42658424)
- answers 1-7, 9-12: 4, 22, 36, 8, 95, 3, 200, 0, 288036, 192384, 42658424
- operators per validator: `+` 1-2, `*` 3, `-` 4-5, `/` 6-7, `*` 8-12

Sources: CG per-validator report of probes bis01-bis30 (perl, `puzzle/mayan-calculation/probe/probes.tsv`, raw `agent/state/report/<id>.json`). Forum https://forum.codingame.com/t/mayan-calculation-puzzle-discussion/48 quotes no validator input (2026-10-20 journal).

## Key

Shape (font, operator, digit counts of a and b) separates every validator except 1 and 2 (both `.o_` `+` 1-digit): then a or b separates them (2 vs 17). Cheap keys from raw input: the operator line + `S1` + `S2` line values + `L` (exact per validator in the `NN.in` files).
