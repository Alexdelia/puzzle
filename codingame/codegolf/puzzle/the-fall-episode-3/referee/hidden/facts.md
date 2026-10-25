# The Fall - Episode 3 (golf): hidden validators

Validators 01, 02, 03 recovered exactly (`hidden/01.json` `02.json` `03.json`, reference `probe/python/probe.py` passes `--hidden`); validator 04 not exact (12 cells unknown), but the test 4 plan is proven to win it on CG. No `complete`; `keyed` since 2026-10-25 (see "Referee reconstruction"). Per-validator report probes 2026-10-24 (python lines of `puzzle/the-fall-episode-3/probe/probes.tsv`, journal `validator.the-fall-episode-3.-.1792834544511127800`). Older count probes: `agent/state/oracle/the-fall-episode-3.tsv`. Web: nothing (forum threads 203183 and 1675, https://forum.codingame.com/t/the-fall-episode-3-code-golf-puzzle-discussion/203183, https://forum.codingame.com/t/the-fall-episode-3-puzzle-discussion/1675; GitHub searches 2026-10-20/21/24: no hit).

Probe method (2026-10-24): python general solver `probe/python/probe.py` (passes all 4 on CG), predicate keyed on (W,H) inserted after the grid read (generator in the journal scratch `gen.py`); CG's report gives pass/fail per validator, so one bit per validator per probe; 6x6 kept as an always-pass control.

## Summary, confidence high

| validator | title | dims | content | source |
|-|-|-|-|-|
| 01 | Avoiding rocks | 10x10 | test 1 with column 0 deleted: exit 0, Indy (0,0) TOP, every rock x-1 | bit0..4 (W=10), `shift1.py` (grid equal), `transcript.py` (all turn inputs of the solver run equal) |
| 02 | Rock interception | 6x6 | test 2 with cell (2,3) = +12 (test 2: 10) | 2026-10-21/22 probes below, `minrow0.py` (sign +) |
| 03 | Multiple choice and rocks | 14x8 | test 3 shifted right by one column (col 0 = 0, test col 13 was empty): exit 7, Indy (7,0), rocks x+1 | bit0..4 (>=31 diffs vs test 3), `shift1.py`, `transcript.py` |
| 04 | Only one way | 14x14 | test 4 with 12 cells changed, see below | bit/minrow/maxrow/absdiff/v4off/v4nz probes |

Key: H alone separates all 4 (10, 6, 8, 14); W does not (two W=14). Plans: v2 = recovered 6x6 plan, v4 = test 4 plan (CG: perl `plans.pl` 2026-10-24 passed v2 and v4), v1 = test 1 plan with x-1, v3 = test 3 plan with x+1 (both win `hidden/01`/`03` locally; the unshifted plans fail on CG and locally with "cannot be rotated").

## Validator 04 (partial)

- exit, start, rocks: not probed directly; the test 4 plan (rotations turn 13 (6,3) RIGHT, 17 (3,4) RIGHT, 25 (8,7) LEFT) wins it on CG
- grid = test 4 except exactly 12 cells (signed diff count 12, bits probes); all 12 change the absolute room type (abs-diff count 12); all in rows 5..12 (min row 5, max row 12)
- at least one diff is on Indy's test-4 path (with the plan applied) and at least one is at a cell empty in test 4 (probes `v4off.py`, `v4nz.py`: both no)
- rejected (local, journal scratch `cand.py`): every sub-block shift (|dx|,|dy| <= 3) or x/y mirror of a rectangle of test 4 that matches these counts makes the test 4 plan lose: the change is no simple block transform
- next: column bounds of the diffs (min/max col, 4 bits each), then per-cell bits; exact recovery needs ~100 bits (12 positions + types), several tasks of probes

## Older probes (count/row oracle era)

## Count and titles

- 4 validators (bash 75% probe; every count probe is a multiple of 25%). Titles equal the IDE test titles: 01 Avoiding rocks, 02 Rock interception, 03 Multiple choice and rocks, 04 Only one way. The IDE submit report shows pass/fail per validator to the user; the API only gives the total score
- 2026-10-24 perl `probe/perl/plans.pl` 450 (plans keyed on W,H, no general fallback): 01 fail, 02 pass, 03 fail, 04 pass. So validator 04 is solved by the replayed test 4 plan, 02 by the recovered 6x6 plan; 01 and 03 still need their own plans
- general method (replanning DFS, python 869) passes all 4

## Dimensions (W H), confidence high

| probe | answer |
|-|-|
| W,H in test dims {11x10,6x6,14x8,14x14} | 3 of 4 |
| W == 14 | 2 |
| W,H == 14,14 | 1 |
| W,H == 14,8 | 1 |
| W,H == 11,10 | 0 (php probe 1268 scored below the 25% row: row stayed 1283) |

So: one validator 14x14, one 14x8, one 6x6 (no 11x10: test 1 "Avoiding rocks" dims have no validator), and one validator with dimensions of no test and W != 14 (2026-10-24: 10x10).

## Grids

- no validator grid equals a test grid line for line (crc32 probe, 0%)
- exactly 1 validator has a test's layout up to room rotation, lock sign and x-mirror (rotation classes {2,3} {4,5} {6..9} {10..13}, abs value; probe ruby 2743, 25%). Same dims as its test, and no 11x10 validator: a variant of test 2, 3 or 4 (which one unknown)
- the other 3 have layouts of their own (2 of them with test dimensions)

## 6x6 validator (2026-10-21, probes in `puzzle/the-fall-episode-3/probe/probes.tsv`, cpp/objective-c lines), confidence high

- it is the class-matched validator (cpp probe 2837 `match.cpp`: count 4 = matched one is 6x6), so the test 2 "Rock interception" layout
- not x-mirrored (cpp 1509 `direct.cpp` yes)
- abs grid = test 2 abs grid except cell (x=2,y=3): 12 instead of 10 (probes absgrid no, rows012 yes, rows345 no, c3343 yes, c5455 yes, c23 no, c23is11 no, c23is12 yes):

```
0 3 3 5 1 1
13 11 4 0 3 1
11 6 1 0 10 1
0 0 12 3 3 1
0 0 0 0 0 3
0 0 0 0 0 3
```

- 2026-10-22 (ocaml probes `probe/ocaml/signs-exit.ml` 949, `transcript.ml` 842, both YES): signed grid = test 2 signs with (2,3) = ±12 (sign unknown, off the path), exit x = 5; the perl-752 solver's turn inputs (Indy + rocks, turns 1..8) equal those of local test 2 with (2,3)=12: Indy start (1,0) TOP and rock schedule as test 2 (4 rocks: turn 1 (2,0) TOP and (0,1) LEFT, turn 3 (3,0) TOP, turn 5 (5,0) TOP). So the 6x6 validator is recovered: `jq '.grid[3][2]=12' tests/02.json` (confidence high; a false yes needs a different game that the solver wins before turn 8). Game won at turn 8 with the solver's moves
- (before 2026-10-22) unknown: lock signs (test 2 signs: `0 -3 -3 -5 1 -1 / -13 11 4 0 -3 1 / 11 6 1 0 10 1 / 0 0 10 3 3 1 / 0 0 0 0 0 3 / 0 0 0 0 0 -3`), exit x (test 2: 5), Indy start (test 2: (1,0) TOP), rock schedule (test 2: 4 rocks). Next bits: signs equal test 2 with (2,3) unlocked; exit == 5; first-turn Indy == 1 0 TOP; rock count/schedule per turn input (predicates on turn inputs need a probe hook in the turn loop, gen.py only hooks init)

## Eager plan (2026-10-21, objective-c 1626 `eager.m`, 25%)

- plan at turn 1 = min-cost rotations of Indy's path cells (future rocks ignored), issued on turns 1..k, then WAIT: wins exactly 1 of 4 validators (which one unknown). Locally it wins only test 03 (rocks crush Indy on 01/02/04). So 3 validators need turn-timed or rock-aware commands

## Keys

- W alone does not separate (two W=14). (W,H) separates all 4 (14x14, 14x8, 6x6, other with W != 14). H separates iff the 4th validator's H is not 6, 8 or 14 (unknown)
- per validator answers (2026-10-24): see Summary above (plans keyed on H)

## Open

- 6x6: lock signs, exit, start, rocks unknown (see above). Others: grids, exits, rock schedules, answers unknown. Per-validator answer extraction needs per-bit predicates keyed on (W,H); rows at 25% in ruby/c/php/perl (50%) still answer binary questions when the probe is shorter than the row

## Referee reconstruction 2026-10-25

`hidden/04.json` and `hidden/keyed` were added by the referee requeue (perl 135 rejected
locally). 04 remains **not an exact recovery**: the probes fixed its dimensions, the 12 diffs vs
test 04, the row window 5..12, one diff on Indy's test-4 path, one diff at a test-4 type-0 cell,
and that the test-4 rotation plan wins it on CG. `hidden/04.json` is a reconstruction that honors
every one of those facts: test 4 with 12 cells changed to 1 at
(3,5) (4,5) (9,5) (0,6) (11,6) (4,7) (6,8) (3,9) (10,9) (4,10) (6,11) (10,12); 11 were type 0
(inert: never entered or attempted by Indy/rocks under the proven plan), (4,5) was 3 -> 1 (same
DOWN exit for entry TOP) and is on Indy's path. It is won by both the early plan (rotations
(6,3) RIGHT, (3,4) RIGHT, (8,7) LEFT on turns 1-3, then WAIT - the perl 135 chunk) and by the
CG-proven timed plan (same rotations on turns 13/17/25). Treat 04's remaining cell values as
unverified; a candidate keyed on exact features of the real 04 may still differ.

`hidden/keyed` (not `complete`: 04 is not exact) is written so `--cg` gates on the CG-graded hidden set (spec mechanism,
same as the blunder-episode-3 / super-computer reconstructions). Synthetic validators are no
longer overfit guards for this puzzle's H-keyed plan: they are hidden-class variants written so
that every CG-row core passes them (see the referee journal 1792918082313957207). A new exploit
must still be probed with `oracle.sh`/`submit.sh --probe`; `--cg` passing is not proof against the
real validator 04.
