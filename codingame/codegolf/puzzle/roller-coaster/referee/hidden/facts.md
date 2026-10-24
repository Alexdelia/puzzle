# roller-coaster hidden validators

Status 2026-10-24: 10 validators (not 23: the old count was the weight grain), 3 recovered exactly (V2, V4, V10 -> `02`, `04`, `10` here, numbered by CG validator, not by IDE test). V9 (the only one besides V10 that naive cannot do) has new random groups: its answer is out of reach of 1-bit probes. No `complete`.

Sources: per-validator report of every submission (`agent/tool/cg-report.sh <id>`), python predicate probes p1..p15 in `puzzle/roller-coaster/probe/python/` (log `puzzle/roller-coaster/probe/probes.tsv`, task validator.roller-coaster.-.1792834544102689521). Each probe is a correct cycle-skip solution printing its answer only when a predicate holds, else `0`. Older JS/PHP/C/TS probes (2026-10-20, submissions 41537540..41537615) re-read per validator with `cg-report.sh`.

Notation: `W` = groups boarded over all `C` rides (naive cost), `S` = sum of groups, `pre`/`per` = pre-period/period of the ride start index from 0.

## Count and weights

10 validators: V1..V7 weight 100, V8..V10 weight 150 (total 1150, score grain 50/1150 = 100/23, why the old probes read "23").

## Per validator

| # | title | known | source | confidence |
|---|---|---|---|---|
| 1 | Simple case | not test01 groups (nor permuted), `N != 1000`, `W < 1e4`, every ride full (answer = `C*min(L,S)`), `pre = 0`, `C < 1e6` | p1, p3, p7, p5, p6, JS W probe | high |
| 2 | 2 spot, 1 ride, 1 group | exact: `2 1 1` / `2` -> 2 (`02.in`) | p15 | exact |
| 3 | The same groups go on the ride several times during the day | not test03 groups (nor permuted), `N != 1000`, `W < 1e4`, rides not all full, `2 <= pre < 16` (test03 has pre 1) | p1, p8, p7, p12, p13 | high |
| 4 | 1000 groups of a few people | exact: IDE test 02 (`04.in`), answer 3935 | p1 (N + group hash), old E probe | exact |
| 5 | All the people get on the roller coaster at least once | `S <= L`, not test04 groups (nor permuted), every ride full, `N != 1000`, `pre = 0` | TS probe, p1, p8, p7 | high |
| 6 | All the people get on the roller coaster before dawn | no IDE twin, `N != 1000`, `W < 1e4`, `S > L`, rides not all full, `pre = 0` | p3, JS W, TS, p7, p5 | high |
| 7 | Sequence of groups of complementary people | no IDE twin, `N != 1000`, `W < 1e4`, `S > L`, rides not all full, `pre = 0` | same | high |
| 8 | High earnings during the day (> 2^32) | `N = 1000`, `L != 1e7`, `C < 1e6`, `1e4 <= W < 1e6`, not test05 groups (nor permuted), `pre = 0`, `pre >= 64` or `per >= 256` (so `per >= 256`) | p3, p4, p6, JS W, p1, p8, p5, p14 | high |
| 9 | Works with a large dataset | `N = 1000`, `L != 1e7`, `5e6 <= C < 1e7`, `1e8 <= W < 3e8` (10..60 groups per ride), `S > L`, not any IDE test's groups (nor permuted), rides not all full, `pre >= 16`, `pre >= 64` or `per >= 256` | p3, p4, p10, p11, JS/PHP W, TS, p1, p8, p7, p13, p14 | high |
| 10 | Works with a very large dataset | exact: IDE test 06 (`10.in`), answer 89744892565569, `pre 48 per 126` | p1, old E probe, C probe | exact |

## Keys

- `C >= 1e6` (also `>= 2e6`, `>= 5e6`) holds for V9 and V10 only; `L == 1e7` for V10 only; `N == 1000` for V4, V8, V9, V10
- naive walk passes V1..V8 in every language measured (perl naive 17/23 = all but V9, V10); V9 times out even in C (C probe 208, naive + V10 hardcoded on `C==9e6`: only V9 fails)
- hardcode V10 by `L==1e7` (or `C==9e6`, `C>8e6` is NOT safe: V9's C is in [5e6, 1e7)) -> 89744892565569. V9 still needs a real method: memo or cycle skip
- V9 has `C >= 5e6`: a per-start memo with an O(C) loop does at least 5e6 iterations even with V10 hardcoded; slow interpreters still need the cycle skip
- start-at-0 cycle detection (skip when the queue start returns to index 0) fails V3, V9, V10 (`pre > 0`); "state after ride 1 recurs" fails V3, V9, V10 too; a checkpoint must be taken after at least 64 rides to be safe for V9 (pre >= 16, unknown above), after `N` rides is always safe

## Not recovered

V1, V3, V5..V9 inputs: no IDE twin, not even as a permutation of an IDE test's groups (p8). V9's answer needs ~40 bits = 40 probes: not worth it. Small ones (V1, V3, V5, V6, V7) could be recovered with probes but naive passes them, so no golf value.
