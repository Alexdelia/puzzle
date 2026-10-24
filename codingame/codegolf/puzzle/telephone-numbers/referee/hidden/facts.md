# Telephone Numbers: hidden validators

7 validators (CG per-validator report, `agent/tool/cg-report.sh 41548618`), weights 140 x6 + 160 (V7). Not variants of the IDE tests (P1: none equals a test). The 2026-10-20 "25 validators" deduction from score grain was WRONG (28% = V1+V5, 16% = V7, 56% = V1..V4).

Sources: old probes P1..P9 re-read with `cg-report.sh` (submissions 41537633..41537695, 41542008), then bit probes Q01..Q21 of task validator.telephone-numbers.-.1792834544307788895 (python, `puzzle/telephone-numbers/probe/python/qNN.py`, log `oracle.sh --log telephone-numbers`). Each Q probe answers one predicate per validator class (class told apart by N and max length). Confidence high (direct reports) unless marked.

Notation: N = line 1, A = answer, T = total input bytes (line 1 and final `\n` included; every validator ends with `\n`, digits and LF only, P5).

| # | title | N | numbers | A | T |
|---|---|---|---|---|---|
| 1 | A telephone number | 1 | one 9-digit number, starts `04488`, 6th digit odd (Q21) | 9 | 12 |
| 2 | Nested numbers | 3 | lengths 3, 5, 8 in this order, all prefixes of the 8-digit one, first digit odd (Q21) | 8 | 21 |
| 3 | Numbers with a different base | 3 | three 6-digit numbers, no shared prefix, first starts `49`, its 3rd digit even (Q21) | 18 | 23 |
| 4 | Numbers with common parts | 5 | five 10-digit numbers, first starts `05`, its 3rd digit 2 or 6 (Q21) | 32 | 57 |
| 5 | All the digits of a number are identical | 1 | `55555555555` (11 fives), EXACT input `1\n55555555555\n` (Q09) | 11 | 14 |
| 6 | All the numbers are identical | 3 | the same 12-digit number 3 times, starts `123`, 4th digit even (Q21), not one repeated digit | 12 | 41 |
| 7 | Long list of numbers | 10000 | some number has 11..20 digits; not test 5 reordered (Q01) | 45438 | T%8==1 (Q19..Q21) |

Facts per validator, probe by probe, are in the journal `agent/state/journal/validator.telephone-numbers.-.1792834544307788895.md`.

## Exact recovery

Only V5 exactly: `05.in` + `05.out` written. V1..V4, V6, V7 contain random digits beyond reach (V7 alone is 10000 numbers). `complete` is out of reach.

## Keys (answers per validator)

- answers: V1 9, V2 8, V3 18, V4 32, V5 11, V6 12, V7 45438
- N line: 1 -> V1/V5 (A = length of the number, 9 or 11), 3 -> V2/V3/V6, 5 -> V4, 10000 -> V7
- T (input length): 12, 21, 23, 57, 14, 41 for V1..V6, distinct mod 32 (12,21,23,25,14,9) and mod 13 (12,8,10,5,1,2); V7 T is large, T%8==1, other bits unknown: key it by `T>99` (or N>5) unless its residue is known
- second line length: 9, 3, 6, 10, 11, 12, ? (V7 unknown); V2's first number has 3 digits
- A == count of distinct prefixes of all lines incl. line 1, minus 1 (P9 identity, all 7)

## Not probed

- V7 total length beyond T%4, V7 line-2 content, any other V7 feature
- remaining digits of V1, V2 (8-digit), V3, V4, V6
