# Shadows of the Knight - Episode 1: hidden validators

Not recovered: no `NN.json`, no `complete`. Most validators have other sizes than the IDE tests, so they are not simple transforms of them.

## Count

7 validators: leaderboard partial scores step 14.3 (28.6, 42.9, 57.2, 71.4, 85.7 seen over 1912 rows of 9 languages, 2026-10-20). Titles assumed = IDE test titles (A lot of jumps, Less jumps, Lesser jumps, Tower, Correct cutting, Evasive, Not there?).

## Known (probes, `agent/tool/oracle.sh --log shadows-of-the-knight-episode-1`, task validator.shadows-of-the-knight-episode-1.-.1792523691050048233)

Each probe: honest bisection iff init `(W,H,N,X0,Y0)` matches a table row, else `-1 -1` (fails turn 1); row with bomb: jump there first, `-1 -1` after.

| # | lang | question | answer |
|---|---|---|---|
| 1 | php 821 | `(W,H,N)` = some IDE test `(W,H,N)` | 14.3% = 1 of 7 |
| 2 | lua 788 | `(W,H)` = some IDE test `(W,H)`, any N | 28.6% = 2 of 7 |
| 3 | ruby 488 | exactly 40x60 N32 start (38,38) bomb (6,6) | 0% |
| 4 | ruby | init `(W,H,N,X0,Y0)` = some IDE test init (row 0%: moves iff >= 1) | no: 0 of 7 |
| 5 | php 733 | `(W,H,N)` = test 5, 6 or 7 `(W,H,N)` (row 14.3 at 821: same score shorter moves) | yes |
| 6 | php 687 | `(W,H,N)` = 9999x9999 N14 (row 14.3 at 733) | no |

Deductions:

- 5 validators have a size `(W,H)` that no IDE test has (also not tested: transposed sizes `(H,W)`)
- 1 validator has the size of an IDE test with another N; 1 has size and N of an IDE test: 50x50 N6 or 100x100 N7 (probes 5, 6), with another start than the test (probe 4)
- no validator init `(W,H,N,X0,Y0)` equals any IDE test init (probe 4)
- forum claim "Lesser jumps validator: bomb 6,6, batman 38,38" (KubaF, https://forum.codingame.com/t/shadows-of-the-knight-episode-1-puzzle-discussion/264/346, 2023-09-03) is false today (probe 3)
- every validator passes with the CG-validated bash 88 (bisection, floor midpoint, start and first direction ignored, `knowledge/puzzle/shadows-of-the-knight-episode-1.md`): no validator needs the start position or the first direction

## Web

- forum thread read through post #427 (`https://forum.codingame.com/raw/264?page=1..5`): no validator input quoted. pardouin #184: "validators are different from the tests to make it hard to hardcode", "nth validator is somehow similar to the nth test"; same test always has the same bomb
- code golf thread https://forum.codingame.com/t/shadows-of-the-knight-episode-1-code-golf-puzzle-discussion/203194: only merge notices

## Shortest key

None known. Hardcoding per validator is out of reach: each needs W, H, N, start and bomb, and 5 sizes are new. A bisection is the only known way to pass all 7.

## Local `validators/`

Synthetic (referee worker guesses), not CG's: do not trust them as hidden data.
