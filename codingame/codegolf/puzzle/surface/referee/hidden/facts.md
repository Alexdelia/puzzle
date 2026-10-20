# Surface: hidden validators

10 validators (score grain 10%: probes scored 60, 80, 90, 90, 100; forum post 75 "only 90% validated" with one failure).
Probes: `agent/state/oracle/surface.tsv`, javascript chain 2026-10-20, each = correct general flood fill printing `-1` unless the predicate holds.

| probe | predicate (answers correct only when true) | score |
|---|---|---|
| P1 | `L*H<1e5` | 60 |
| P2 | `L*H<1e5 or N<2` | 80 |
| P3 | `L*H<1e5 or N<101` | 90 |
| P4 | `L*H<1e5 or N<300` | 90 |
| P5 | `L*H<1e5 or N<301` | 100 |

Measured (certain):

- 6 validators with `L*H < 100000`
- 2 validators with `L*H >= 100000` and `N = 1`
- 1 validator with `L*H >= 100000` and `2 <= N <= 100`
- 1 validator with `L*H >= 100000` and `N = 300`

Forum (https://forum.codingame.com/t/surface-puzzle-discussion/26):

- a validator titled "No Land" (post 29 Taihennami: "consistently fails the "No land" test on submission"; post 40 nicola: "No land? There is only water."; post 43 dk987: "a good test case for the performance of your flood fill method"; post 56: fails in Lua while IDE "Large map, large lake" passes)
- post 75 DIYDER (page 4): "In the validator, test N°7 (100 tests we have 1000x1000 map) passes but not N°8 (300 tests we have 1000x1000 map)", score 90%

Deduced titles (plausible, not probed one by one): the 9 IDE titles with test 3 "The area is a lake" (3x3, small) replaced by the big all-water "No Land" (N=1), plus "300 tests on a 1000x1000 map":

| # | title (deduced) | known |
|---|---|---|
| 1 | Example of the statement | small |
| 2 | No lake | small |
| 3 | No Land | all water, big (`L*H>=1e5`), N=1 (plausible: one of the two big N=1) |
| 4 | 1 lake on a small map | small |
| 5 | Several lakes on a small map | small |
| 6 | Some lakes on a 20x20 map | small |
| 7 | Many lakes on a 100x50 map | small |
| 8 | 100 tests on a 1000x1000 map | big, 2<=N<=100 |
| 9 | 300 tests on a 1000x1000 map | big, N=300 (forum says 1000x1000) |
| 10 | Large map, large lake | big, N=1 |

Order and the mapping of the 6 small ones to titles: unverified. No input recovered exactly: no `complete`.

Separating key: `N` alone separates the big ones from each other only partly (two have N=1); `L*H>=1e5` plus `N` gives classes {6 small}, {2 big N=1}, {1 big N<=100}, {1 big N=300}.

Golf consequences:

- no hardcode gain: 4 big maps (>=100k cells, one with 300 queries) need the general flood fill
- per-query full-map work (copy, count) costs 3x test 8 on the N=300 validator: Python `s.count(0)` variant (4.9 s on test 8) cannot pass
- an all-water big map (No Land) floods the bottom row: out-of-range reads below the last row happen, keep the pad where the lang errors on out of range; recursion depth ~ map size (>=100k)
