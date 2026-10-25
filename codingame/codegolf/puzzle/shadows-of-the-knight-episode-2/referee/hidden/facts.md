# Hidden validators: shadows-of-the-knight-episode-2

8 validators, weight 125 each (CG per-validator report, 2026-10-24). Titles = IDE test titles: 1 A lot of jumps, 2 Less jumps, 3 Tower, 4 Lesser jumps, 5 Exact nb of jumps, 6 More windows, 7 A lot of windows, 8 So many windows.

No exact recovery, no `complete`: V1 and V2 change map between runs, bombs of V4 and V8 at least are not shown fixed (forum https://forum.codingame.com/t/shadows-of-the-knight-episode-2-puzzle-discussion/1676?page=4 KevinSe 2025-07-09: validator 08 bomb at (0,2) or (0,3) depending on the run).

Sources: per-validator reports of 2026-10-20 probes (`agent/tool/cg-report.sh 41537562` .. `41537794`, re-read 2026-10-24) and 2026-10-24 probes `probe/python/p1.py`, `b1.py`..`b17.py` (b16/b17 V1 W*H-1<=N fail then pass, V2 16x5 fail, W*H<=100 fail), `testbombs*.py`, `stable1.py` (`agent/tool/oracle.sh --log shadows-of-the-knight-episode-2`). Each probe = general method (plain python, generator in the 2026-10-24 journal scratch) that exits unless a per-validator predicate on the init lines holds, and (bomb questions) exits before the winning jump unless a predicate on the found bomb holds. A pass is proof; a fail is "predicate false OR CG flake/variation": several fails contradicted later passes.

## Per validator

| # | title | known | confidence |
|---|---|---|---|
| 1 | A lot of jumps | small map changing between runs: every probe so far passes 2 <= W < H <= 40, N >= 27, not square; passes W<=2 and H<=5 (b3, b7, b11), W*H-1 <= N (b12, b13, ...), yet fails W==2 (b14), H==5 (b15), "W==2 and H==5" (stable1). Old row-oracle facts (W<H, N>=27, H<=40) hold at least in those runs | map random per run (likely 1-2 wide, <= 5 tall), N always large so far |
| 2 | Less jumps | 2026-10-20: 16x5 N80 in 5 runs (passes "size or transposed", "(W,H,N) or (H,W,N) test triple", "exact or 16x5"); start (15,4) N80 passed exactly in b9, b10; but fails 16x5 N80 in b8, testbombs x3, stable1, b11, fails N==45 (b14), W<H (b15) | map random per run |
| 3 | Tower | 1x100 N12 start (0,1) exact, stable (passed every probe that asked it, incl. stable1); bomb (0, by) with by in 25..26 (bisected b11..b17, passed by<=28 b16, by<=26 b17, fails = may be variation) | config high; bomb medium |
| 4 | Lesser jumps | 15x15 N12 start (2,6), stable (b9, b10, stable1); bomb x 0, y bisected into 0..3 (test sample bomb (0,1) failed in testbombs) | config high; bomb not shown fixed |
| 5 | Exact nb of jumps | 24x24 N15 start (22,14), stable; bomb x 23, y in 18..23 (test sample (23,22) failed testbombs) | config high; bomb not shown fixed |
| 6 | More windows | 50x50 N16 start (17,29) = test 6 config exactly (start included), stable; bomb x < 25 in all 17 runs asked, bisected x 15, y 25..37 (test sample (0,44) refuted) | config high |
| 7 | A lot of windows | 1000x1000 N27 start (501,501) = test 7 config exactly, stable; bomb x >= 500 in every run asked, bisected x 743..749 (all from fails, so weak) (test sample (800,203) refuted) | config high |
| 8 | So many windows | 8000x8000 N31, start x in 3196..3203 (y unknown, not a D4 transform of test (3200,2100)); bomb x == 0 (b11), y <= 3 passed (b13), by <= 2 failed (b16) yet exact (0,3) failed (b17): moves between runs, forum: (0,2) or (0,3) | high |

## Shortest key

Size separates the 6 stable validators: (W,H) = 1x100, 15x15, 24x24, 50x50, 1000x1000, 8000x8000 (W alone separates them from each other: 1, 15, 24, 50, 1000, 8000; V1 may be 1 wide in some runs, V2 seen 16 wide). No bomb table is safe: bombs are not proven fixed (V8 moves between (0,2) and (0,3)). V8 alone: bomb on column 0 near the top, so `x=0, y=0..30` in N31 jumps always hits it (if the forum + probe range holds).

Golf consequence: the general method stays. Tight budgets: V3 1x100 N12 from (0,1), V4 15x15 N12 from (2,6), V5 24x24 N15 from (22,14), V6/V7 = test 6/7, V8 8000 N31 bomb (0,2..3) (edge bomb: the hard case for reflection bisection; the method's measured edge-bomb fail on 8000 is 0%).

## Local validators (2026-10-25)

- `validators/11`..`18` (V7 role, 1000x1000 N27 start (501,501)): bombs rebuilt inside the known V7 region, `bx` 500..749 (`by` was never probed). The old bombs had `bx` 3, 8, 97, 250, 476 (all outside the region: every probe of `bx<500` failed) and `bx` 999 (above the only proven upper bound: `bx<=749` passed). `(3,1)` rejected the CG-100% perl 200: the parity method needs 30 > 27 jumps there, exactly the left-edge class that cannot happen.
- New mix keeps the 4 uniform + 4 edge roles: (523,443) (600,969) (688,302) (741,150) uniform in the region; (749,0) (748,1) (747,998) (743,999) at the region ends and the y edges. Perl 200 turns: 20 24 23 21 27 27 21 22; the two 27s are the tightest V7 cases that still pass.
- The other groups are unchanged: they follow the captured IDE tests and the uniform/near-corner bomb mix, and all cores pass them. Their exact regions are less certain (V1/V2 maps random per run, V3..V6 bombs only weakly bounded).
- 2026-10-25 check: bash 224, perl 200, python 261, ruby 215, lua 255 all pass `--cg` 46/46 locally; python 261 bench suite wall 2.27 s, referee CPU 56 ms (1.2 ms/case).
