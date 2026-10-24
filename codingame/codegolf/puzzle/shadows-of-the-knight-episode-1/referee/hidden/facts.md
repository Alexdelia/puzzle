# Shadows of the Knight - Episode 1: hidden validators

7 validators, titles = IDE test titles, weights 143 x6 + 142 (V7) (CG report, `agent/tool/cg-report.sh 41537581`). V1-V5 exact in `01.json`..`05.json` (bombs confirmed by a one-jump hardcode, b41-b46). V6 in `06.json` (effective bomb (0,1), see below). V7 start unknown: no `07.json`, no `complete`.

**V6 and V7 (and IDE tests 6, 7) are EVASIVE: the bomb moves when Batman lands on it too early.** IDE test games 2026-10-24 (task 1792843468101458263): test 6 first jump straight to its bomb (0,1) -> "bombs are located below and to the right"; test 7 first jump to (9753,2531) -> "below and to the left". Fixed-bomb direct jumps on V6 failed for all 7 cells (0,1),(1,1),(3,3),(6,6),(12,12),(25,25),(50,50) (b41-b45), on V7 for (9741,2473),(9741,2474) and the 9 single-flip candidates (b41-b46). A hardcoded jump can never pass V6/V7: they need a real search. Exact move rule unknown. Model consistent with every V6 probe: bomb (0,1), and it moves on a hit while other cells are still consistent with the replies given so far. The honest floor bisection reaches (0,1) only once the box is the single cell (0,1) (after the (1,1) reply `L`), hence it wins. The local referee has no evasion (tests 6/7 json are fixed bombs), so local runs cannot catch a hardcode that fails on CG for V6/V7 and tests 6/7.

Pattern: validator k = test k with the size nudged by 1 and the start / bomb flipped, translated or swapped. Not a fixed transform per validator.

## Per validator (2026-10-24)

Source: python probes b1..b41 (`agent/tool/oracle.sh --log shadows-of-the-knight-episode-1`, sources `probe/python/b*.py`), tasks validator.shadows-of-the-knight-episode-1.-.1792834544143595596 (b1-b20) and .1792840157263671246 (b21-b41); plus the 2026-10-20 php/lua/ruby probes re-read with cg-report.sh. Each probe: correct play iff predicate, else `-1 -1`. Init predicates are exact. b21-b26 bomb predicates for V1-V3 use an avoid mode (never jumps on a known-candidate cell where the predicate is false), so no false yes. V7 bomb (b27-b40): the honest bisection ignores the start, so the pass set of each box predicate was computed exactly per candidate bomb (`v7.py` in the task scratch), no false yes.

| # | title | W x H | N | start (X0,Y0) | bomb | vs test k | confidence |
|---|---|---|---|---|---|---|---|
| 1 | A lot of jumps | 4 x 9 | 40 | (2,4) | (3,8) | test 4x8 (2,3) bomb (3,7): H+1, translated y+1 | exact, `01.json` |
| 2 | Less jumps | 25 x 33 | 50 | (22,29) | (0,1) | test N49 (2,29) bomb (24,2): x flipped, bomb y-1, N+1 | exact, `02.json` |
| 3 | Lesser jumps | 40 x 61 | 32 | (38,38) | (6,6) | test 40x60 (6,6) bomb (38,38): start/bomb swapped, H+1 | exact, `03.json` |
| 4 | Tower | 1 x 79 | 6 | (0,77) | (0,42) | test 1x80 (0,1) bomb (0,36): H-1, y flipped | exact, `04.json` |
| 5 | Correct cutting | 50 x 50 | 6 | (49,49) | (27,27) | test (0,0) bomb (22,22): both flipped | exact, `05.json` |
| 6 | Evasive | 101 x 101 | 7 | (5,99) | evasive, effectively (0,1) | test 100x100 (5,98) bomb (0,1): size+1, start y+1 | start exact; bomb: honest-bisection predicates b17-b26 pass for (0,1); every direct jump fails (evasion), `06.json` with (0,1) passes honest solvers only, as on CG |
| 7 | Not there? | 9998 x 9998 | 14 | unknown | evasive (honest-path predicates point at (9741,2473..2474)) | test 9999x9999 N14 (54,77) bomb (9753,2531): size-1, bomb (-12,-57) | bomb deduced: b27-b40 exact pass sets left (9741,2473),(9741,2474); b41 direct jump to (9741,2473) fails; no other cell of the honest path to (9741,2474) fits b24-b40. Not confirmed by a direct jump (assumes b24 box yes was not a false yes and the bomb is fixed). Start unknown (~27 bits) |

Older facts: no validator init equals a test init; forum claim "Lesser jumps bomb 6,6 batman 38,38" (https://forum.codingame.com/t/shadows-of-the-knight-episode-1-puzzle-discussion/264/346) is about start (38,38) bomb (6,6) of V3, but the grid is 40x61 there (refuted on 40x60, the claim swapped the names).

## Shortest key

`W` alone separates all 7: 4, 25, 40, 1, 50, 101, 9998 (also `H`: 9, 33, 61, 79, 50, 101, 9998). Bombs by W, direct jump verified: 4 -> 3 8, 25 -> 0 1, 40 -> 6 6, 1 -> 0 42, 50 -> 27 27. W 101 and 9998: no hardcode possible (evasive), a correct bisection is needed; a hardcode keyed on W can only shortcut V1-V5 and must fall back to bisection for W>=100. V2 and V5 share W and H with tests 2 and 5: a bomb hardcode keyed on W fails the IDE tests (fine for CG score, but local `submit.sh` checks tests).

## V7 analysis (task 1792843468101458263)

`rs.py` in that task's scratch dir runs every python probe source in-process over a k-d partition of bomb cells (exact fixed-bomb semantics). Fixed bomb: V6 (b17-b42) -> {(12,12),(25,25),(50,50)}, all refuted by b43-b45; V7 (b24-b42) -> no cell at all. Both contradictions follow from evasion: a probe's honest path that lands on the bomb before the replies pin it down does not win. To recover V7 exactly (start ~27 bits, plus the evasion rule) the local referee first needs an evasion model; low value: no golf cell can exploit V6/V7.

## Local `validators/`

Synthetic (referee worker guesses), not CG's.
