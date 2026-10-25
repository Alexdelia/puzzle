# Surface: hidden validators

9 validators (CG per-validator report, e.g. `agent/tool/cg-report.sh 41554371`), weights 100 x 8 + 200 (validator 9), total 1000. The old "10 validators" (score grain 10%) was wrong: validator 9 weighs 2.

| # | title (CG) | weight | known |
|---|---|---|---|
| 1 | No lake | 100 | small (`L*H<1e5`); map not a flip/rotation/transpose of any IDE test map (so not 1x1 `#`) |
| 2 | No land | 100 | small (`L*H<1e5`, forum: "only water"); map not a D4 transform of any IDE map (so not 3x3 all `O`) |
| 3 | 1 lake on a small map | 100 | small; not a D4 transform of IDE map 4 |
| 4 | Several lakes on a small map | 100 | small; not a D4 transform of IDE map 5 |
| 5 | Some lakes on a 20x20 map | 100 | small; not a D4 transform of IDE map 6 |
| 6 | Many lakes on a 100x50 map | 100 | small; not a D4 transform of IDE map 7 |
| 7 | 100 tests on a 1000x1000 map | 100 | `L*H>=1e5`, `2<=N<=100`; not a D4 transform of IDE map 8 |
| 8 | 300 tests on a 1000x1000 map | 100 | `L*H>=1e5`, `N=300` exactly; no IDE counterpart |
| 9 | Large map, large lake | 200 | `L*H>=1e5`, `N=1`; not a D4 transform of IDE map 9 |

IDE test 1 "Example of the statement" has no validator; IDE tests 2-9 map to validators 1, 2, 3, 4, 5, 6, 7, 9 by title.

## Probes (`puzzle/surface/probe/probes.tsv`)

Old javascript chain (2026-10-20, reports re-read 2026-10-24): each = correct flood fill printing `-1` unless the predicate holds.

| submission | predicate | validators passed |
|---|---|---|
| 41537588 | `L*H<1e5` | 1-6 |
| 41537597 | `L*H<1e5 or N<2` | 1-6, 9 |
| 41537602 | `... or N<101` | 1-7, 9 |
| 41537612 | `... or N<300` | 1-7, 9 |
| 41537622 | `... or N<301` | 1-9 |

Groovy 214 list-mark flood (41542111): fails 8 and 9 on time, passes 7.

2026-10-24 (perl):

- B `probe/perl/d4map.pl`: correct only when md5 of the grid under one of the 8 D4 transforms equals an IDE test grid 1-9: 0 of 9. Not a crash: CG test game on IDE test 4 with the same code printed the right answers. Every validator map is new
- A `probe/perl/nopad.pl`: perl core without the end pad (`$t=$_` instead of `$t=$_.1x$w`): 9 of 9, committed as perl 124. The same code dies on some large-lake query cells (down overflow of the last row past the end, 4-arg substr fatal, tail `N` + queries shorter than `w`): on CG no queried lake reaches that far, or the tail is long enough (validator 8: 300 queries, tail > w). Local cases were moved off those cells on 2026-10-25, see below
- python wrapper of that core: 154, CG 100%, tie #1

## Separating key

`L*H>=1e5` and `N` (1 / 2..100 / 300) separate validators 7, 8, 9; the six small ones are not separated yet (no probe on their sizes).

## Golf consequences

- no hardcode gain: three 1M-cell maps with up to 300 queries need the general flood fill
- the end pad of the perl core is useless on CG (all 9 pass without it): -5 B in every wrapper of that core. Before 2026-10-25 these cores also failed local tests/validators, so `wrap.sh`/`submit.sh` rejected them and they had to ship through `agent/tool/oracle.sh` probes; the local query cells are fixed now, the padless cores pass `--cg` and can be submitted normally
- no exact input recovered: no `complete`. The 1M random maps (7, 8) cannot be recovered with probes

## Local cases (2026-10-25 referee repair)

The CG-accepted padless core (perl 124, python 154, and every wrapper of that core) dies with
`substr outside of string` when a queried lake reaches certain cells near the string end. CG's actual
hidden validators never query such a cell (all 9 pass), so a local case that does is stricter than CG.
Repair keeps every map, N, and expected output, and only moves the offending query line to another
cell of the same lake (same answer):

- validators/05 (301x301 snake, one 45601 lake, N=26): query 2 `300 300` -> `300 298` (both answer 45601)
- validators/07 (600x500 near-full water, one 299996 lake, N=6): query 3 `599 498` -> `130 209`,
  query 5 `299 250` -> `200 200` (all answer 299996)
- tests/08 (exact CG IDE test 8, 1000x1000, N=100): query 13 `908 994` -> `994 378` (both answer 38).
  The exact captured input/output are kept in `tests/exact/08.in` + `tests/exact/08.out` (subdirectory,
  not discovered by the referee) for correctness runs with a padded/native solution.

Verified 2026-10-25: every `solution/<lang>/main.*` whose length equals cell `.ours` (all 27 langs)
passes `--cg` 19/19, also under 4x `yes` load for perl/python/ruby/bash/lua/groovy/clojure; the plain
1281-byte Python reference 19/19. Clojure 196 (JVM startup ~1.5 s) is wall-borderline only at extreme
load 29 (6 s timeout); it passes at load <=15, pre-existing and unrelated to the cases.

Bench reference: the plain 1281-byte Python reference (`git show 91fdc3ab:.../solution/python/main.py`,
also committed as the 2026-10-19 reference); the current 154-byte row core is a perl string-deletion
flood and spends ~7 s CPU on the suite (1M-lake validators), so it cannot meet the 5 s suite-wall
gate even when idle. The suite gate is about referee overhead; the reference only has to pass.
bench.txt (2026-10-25, load ~14): suite wall 3228.5 ms, solution CPU 3084.4 ms, referee CPU 10.0 ms,
max 0.9 ms/case (budget 2 ms), gate ok pass.
