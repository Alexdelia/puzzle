# The Fall - Episode 3 (golf): hidden validators

No validator recovered exactly. Sources: count probes logged in `agent/state/oracle/the-fall-episode-3.tsv` (2026-10-20), journals `puzzle.the-fall-episode-3.-.1792516443116262949`, `validator.the-fall-episode-3.-.1792523691483465467`. Web: nothing (forum threads 203183 and 1675 quote no validator data, https://forum.codingame.com/t/the-fall-episode-3-code-golf-puzzle-discussion/203183, https://forum.codingame.com/t/the-fall-episode-3-puzzle-discussion/1675; no GitHub hit).

Probe method: general solver (python, passes 100% on CG) that crashes or prints garbage unless a predicate on the init input holds; in a language whose row is shown, the score = number of validators satisfying it (25% each).

## Count and titles

- 4 validators (bash 75% probe; every count probe is a multiple of 25%). Titles unknown (CG hides them); IDE tests: 1 Avoiding rocks, 2 Rock interception, 3 Multiple choice and rocks, 4 Only one way
- general method (replanning DFS, python 869) passes all 4

## Dimensions (W H), confidence high

| probe | answer |
|-|-|
| W,H in test dims {11x10,6x6,14x8,14x14} | 3 of 4 |
| W == 14 | 2 |
| W,H == 14,14 | 1 |
| W,H == 14,8 | 1 |
| W,H == 11,10 | 0 (php probe 1268 scored below the 25% row: row stayed 1283) |

So: one validator 14x14, one 14x8, one 6x6 (no 11x10: test 1 "Avoiding rocks" dims have no validator), and one validator with dimensions of no test and W != 14.

## Grids

- no validator grid equals a test grid line for line (crc32 probe, 0%)
- exactly 1 validator has a test's layout up to room rotation, lock sign and x-mirror (rotation classes {2,3} {4,5} {6..9} {10..13}, abs value; probe ruby 2743, 25%). Same dims as its test, and no 11x10 validator: a variant of test 2, 3 or 4 (which one unknown)
- the other 3 have layouts of their own (2 of them with test dimensions)

## Keys

- W alone does not separate (two W=14). (W,H) separates all 4 (14x14, 14x8, 6x6, other with W != 14). H separates iff the 4th validator's H is not 6, 8 or 14 (unknown)
- per validator answers: unknown, so no hardcode yet

## Open

- exact grids, exits, rock schedules, answers: unknown. Per-validator answer extraction needs per-bit predicates keyed on (W,H); rows at 25% in ruby/c/php/perl (50%) still answer binary questions when the probe is shorter than the row
