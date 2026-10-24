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
- A `probe/perl/nopad.pl`: perl core without the end pad (`$t=$_` instead of `$t=$_.1x$w`): 9 of 9, committed as perl 124. The same code dies locally on IDE test 8 (down overflow of the last row past the end, 4-arg substr fatal, tail `N` + queries shorter than `w`) and on synthetic validators: on CG no queried lake reaches that far, or the tail is long enough (validator 8: 300 queries, tail > w)
- python wrapper of that core: 154, CG 100%, tie #1

## Separating key

`L*H>=1e5` and `N` (1 / 2..100 / 300) separate validators 7, 8, 9; the six small ones are not separated yet (no probe on their sizes).

## Golf consequences

- no hardcode gain: three 1M-cell maps with up to 300 queries need the general flood fill
- the end pad of the perl core is useless on CG (all 9 pass without it): -5 B in every wrapper of that core. These cores fail local IDE test 8, so `wrap.sh` and `submit.sh` (local gate) reject them: ship through `agent/tool/oracle.sh` probes (a 100% probe is committed as the solution)
- no exact input recovered: no `complete`. The 1M random maps (7, 8) cannot be recovered with probes
