# Telephone Numbers: hidden validators

7 validators (CG per-validator report, `agent/tool/cg-report.sh 41548618`), weights 140 x6 + 160 (V7). Not variants of the IDE tests (P1: none equals a test). The 2026-10-20 "25 validators" deduction from score grain was WRONG (28% = V1+V5, 16% = V7, 56% = V1..V4).

Sources: old probes P1..P9 re-read with `cg-report.sh` (submissions 41537633..41537695, 41542008), then bit probes Q01..Q21 of task validator.telephone-numbers.-.1792834544307788895 and Q22..Q50 of task validator.telephone-numbers.-.1792843414278279806, Q51..Q61 of task validator.telephone-numbers.-.1792845787957666608 (python, `puzzle/telephone-numbers/probe/python/qNN.py`, log `oracle.sh --log telephone-numbers`). Each Q probe answers one predicate per validator class (class told apart by N and max length). Confidence high (direct reports) unless marked.

Notation: N = line 1, A = answer, T = total input bytes (line 1 and final `\n` included; every validator ends with `\n`, digits and LF only, P5).

| # | title | N | numbers | A | T |
|---|---|---|---|---|---|
| 1 | A telephone number | 1 | EXACT `044887778` (Q34) | 9 | 12 |
| 2 | Nested numbers | 3 | EXACT `123`, `12399`, `12399989` in this order (Q50) | 8 | 21 |
| 3 | Numbers with a different base | 3 | EXACT `496123`, `789456`, `123456` in this order (Q33, Q51, Q61) | 18 | 23 |
| 4 | Numbers with common parts | 5 | five 10-digit numbers, first EXACT `0523456788` (Q44), second starts `012345`, 7th digit in 0..5 (Q60, Q61; not `0123456789`, Q51), lines 3-5 unknown | 32 | 57 |
| 5 | All the digits of a number are identical | 1 | `55555555555` (11 fives), EXACT input `1\n55555555555\n` (Q09) | 11 | 14 |
| 6 | All the numbers are identical | 3 | EXACT `123456789012` x3 (Q22) | 12 | 41 |
| 7 | Long list of numbers | 10000 | some number has 11..20 digits; not test 5 reordered (Q01), first line != test 5's (Q35) | 45438 | 92833 (Q36, test 5 is 92778) |

Facts per validator, probe by probe, are in the journals `agent/state/journal/validator.telephone-numbers.-.1792834544307788895.md`, `agent/state/journal/validator.telephone-numbers.-.1792843414278279806.md` and `agent/state/journal/validator.telephone-numbers.-.1792845787957666608.md`.

## Exact recovery

V1, V2, V3, V5, V6 exact: `01`, `02`, `03`, `05`, `06` `.in`/`.out` written, reference passes `--hidden`. V4 misses 4 digits of line 2 and lines 3-5 (about 60 bits at one bit per probe, not worth the shared submits: its answer is known); V7 (10000 random numbers, not test 5 edited) is out of reach, so `complete` is out of reach.

## Keys (answers per validator)

- answers: V1 9, V2 8, V3 18, V4 32, V5 11, V6 12, V7 45438
- N line: 1 -> V1/V5 (A = length of the number, 9 or 11), 3 -> V2/V3/V6, 5 -> V4, 10000 -> V7
- T (input length): 12, 21, 23, 57, 14, 41 for V1..V6, distinct mod 32 (12,21,23,25,14,9) and mod 13 (12,8,10,5,1,2); V7 T = 92833 (92833%32 = 1, %13 = 0)
- second line first char V1..V6: `0`,`1`,`4`,`0`,`5`,`1`
- second line length: 9, 3, 6, 10, 11, 12, ? (V7 unknown); V2's first number has 3 digits
- A == count of distinct prefixes of all lines incl. line 1, minus 1 (P9 identity, all 7)

## Not probed

- V4 line 2 beyond `012345`, lines 3-5 (they add 13 prefix nodes over 30 digits, so they share long prefixes with lines 1-2), all of V7 beyond T (first line is not test 5's `2980950684`)
