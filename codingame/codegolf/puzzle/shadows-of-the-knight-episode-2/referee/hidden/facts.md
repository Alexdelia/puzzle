# Hidden validators: shadows-of-the-knight-episode-2

Exact recovery impossible by construction: bomb position is drawn per run ("For some tests, the bombs' location may change from one execution to the other", statement; forum https://forum.codingame.com/t/shadows-of-the-knight-episode-2-puzzle-discussion/1676?page=4 KevinSe 2025-07-09: validator 08 bomb at (0,2) or (0,3) depending on the run; page 3: "hardcoded three sets of coordinates, the bomb moved to a 4th"). No `complete` marker will ever be valid; no bomb hardcode possible. What can be known: per-validator W H N X0 Y0.

Probes: `agent/tool/oracle.sh --log shadows-of-the-knight-episode-2` (2026-10-20). Each probe = the general method (knowledge "Shortest known", ~1% random fail on 1000/8000 maps) printing garbage unless a predicate on the init lines holds. A fresh language shows exact pass share, later probes in the same language show it only when higher.

## Count

8 validators (V multiple of 8 from the 87.5% probe, 8 IDE tests with titles). Confidence high.

## Configs (multiset; validator numbering not resolved)

| what | validators | source |
|---|---|---|
| (W,H,N,X0,Y0) identical to some test | 2 | JS probe 25% |
| (W,H) identical to some test size | 6 | TS probe 75% |
| (W,H) = 16x5 (test 1 "A lot of jumps" 5x16 transposed) | 1 | TS 87.5 with {16x5,32x18}, PHP row moved with {16x5} only |
| (W,H) or (H,W) is a test size, N equal to that test's N | 7 of 7 (so 16x5 has N80, the 6 exact sizes keep their test N) | Lua probe 87.5% |
| odd one: size not a test size nor transposed | 1 | PHP 87.5% |

Odd validator features: not square (JS 87.5), W<H (TS W>H hidden), neither side 1 (Lua hidden), max(W,H) < 1000 (PHP hidden), N >= 27 (JS 100%, committed), N != 45 (Lua hidden), N != 80 (PHP hidden), H <= 40 (TS 100%, committed). So 2 <= W < H <= 40, N >= 27 and not 45/80: budget >= 27 vs <= 12 bisection jumps, never tight.

Most likely (unverified): 6 exact sizes = 6 of the 7 test sizes 18x32, 1x100, 15x15, 24x24, 50x50, 1000x1000, 8000x8000 (distinctness assumed, not probed); 5 of these 7 validators have a start other than the test start; the odd one replaces one test size.

## Shortest key

None useful: bombs random, so no key -> answer table. Golf consequence: every validator has a test size (or 16x5) with the test N, except one small map (<1000, N >= 27, generous). Budget-tight validators are the test-size 1000x1000 N27 / 8000x8000 N31 ones (if present, not separately probed) with random bombs: the general method's ~1% fail stays.
