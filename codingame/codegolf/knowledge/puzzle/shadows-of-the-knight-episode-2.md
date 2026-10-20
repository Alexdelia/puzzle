# Shadows of the Knight - Episode 2

pretty_id `shadows-of-the-knight-episode-2`, leaderboard `shadows-of-the-knight-episode-2-codesize`, interactive.

## Rules (as referee)

- init lines `W H`, `N`, `X0 Y0`, then each turn one line `UNKNOWN` (turn 1) or `WARMER` / `COLDER` / `SAME`
- answer `X Y`: jump there, compare squared Euclidean distance to bombs before/after: `<` WARMER, `>` COLDER, `=` SAME
- win: jump lands on bombs. lose: N jumps done without landing (win on jump N counts, CG test 6 won at jump 16 of 16)
- local referee rejects out-of-building windows and malformed lines (CG behavior for those not probed)

## Tests (CG, 2026-10-19)

- 01 `5 16` N80 start (1,5); 02 `18 32` N45 (17,31); 03 `1 100` N12 (0,98); 04 `15 15` N12 (3,6); 05 `24 24` N15 (22,13); 06 `50 50` N16 (17,29); 07 `1000 1000` N27 (501,501); 08 `8000 8000` N31 (3200,2100)
- bombs never shown in the log: learned from the winning window only
- bombs move between runs on tests 3, 5, 6, 7 (seen twice with different values); test 8 seen near (0,0..12) twice. tests/*.json hold one sample each, validators/ hold random bombs (center and near corners) for the same W H N start
- jump budgets are tight: plain reflection binary search with clamp to the edge loses test 7 and 8 (clamped jump from an edge gives a threshold outside the interval, wasted jumps). Edge bombs on 8000x8000 are the hard case

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout), same as statement

## Reference

- `solution/python/main.py` 1793 chars, 100% on CG: per axis interval, 3-ply minimax over a few candidate jumps; second axis starts with x fixed and uses the exact half-plane `2B.(P-Q) < |P|^2-|Q|^2` with the known x
- once one coordinate is known, any jump (both coordinates changing) still gives a 1D threshold on the other: no wasted transition jump needed

## Shortest known

Method (verified: local referee 46/46 tests+validators, python 277 chars; failure probabilities from an exact per-axis sim over every bomb cell): per-axis reflection bisection, x then y, no minimax.

- loop `for i in x,y`: `l=0`, `h=S-1` (S = W or H), `while l<h`: jump, read, update; after the loop set coordinate i to `l`, print, read (and ignore) one line. For x this is one extra "move onto the known column" jump (always spent, even if already there); for y the same print is the winning jump. No special transition code, response after a jump always belongs to the axis that moved
- jump: `q = l+h-p` (p = current coordinate on axis i). If `q` outside `0..S-1`: `q = (p + S*(q>0)) >> 1` (halfway from p to the edge it overshot, `p/2` or `(p+S)/2`). Then `q -= q==p` (reflection landed on p: p is the interval center)
- update, `s = p+q`, response first letter: `S` -> `l=h=s/2`; else if `(r=='W') == (q>p)` -> `l = s/2+1` (floor); else `h = (s-1)/2` (floor). No `max`/`min` with the old bounds needed
- input: line 2 (`N`) ignored, start `X0 Y0` needed (it is p), line 4 `UNKNOWN` read and dropped before the loop
- needs: integer floor division (or `>>1`, all values non-negative), 2-element mutable position (array or two vars + axis switch), first-char compare of the response. No regex, eval, big ints

Fail probability (x+y jumps > N): budgets are tight, no simple rule is worst-case safe (1000x1000: 14+14 > 27 for every rule tried). This method: test 7 (1000x1000) 0.65% uniform bombs / 0.2% bombs within 16 of edges, test 8 (8000x8000) 0.16% / 0%, other tests 0. So one submit fails with ~1% chance: resubmit if a validator fails on jump count.

Variants measured (sum of fail prob over 8 configs, uniform + edge-bomb, lower is better; method above = 0.010):

- out-of-range rule: `(l+h)/2 - overshoot/2` (C) and `(clamp(q)+p+1)/2` (D1) also ~0.010-0.018; `(clamp(q)+p)/2` 0.014; jump to center `(l+h)/2` 0.35 (62-80% fail on 8000 edge bombs with loose bounds); plain clamp to edge: 24+23 jumps worst case on 8000, hopeless; mirror `abs(q)` 0.65 fail on 8000 edge bombs
- fix of `q==p`: `q-1` = `q^1` 0.010 < `q+1` 0.014; no fix 2.3 (fails small grids)
- merging the q==p fix into the out-of-range branch: 0.25-1.0, rejected
- inclusive bounds (`l=ceil(s/2)`, `h=floor(s/2)`, SAME falls in a branch for free): 0.68-0.78 (8000 edge bombs 62%), rejected: interval stops shrinking
- SAME one-sided (only one bound): 0.33-0.40 with max/min, 2.8-3.1 without, rejected. SAME must set both bounds
- `max(l, ...)`/`min(h, ...)` on the bound update: no gain (0.068 vs 0.014 with the D rule), drop it
- y first: 0.038 vs 0.010, x first

## bash 224 (2026-10-20)

Same method as above, tie rank #1 on CG. Bash specifics in `lang/bash/` (init-slurp-head, response-word-as-arith-var). Refuted by sim: forcing t+q odd so SAME never occurs (fail sum 0.5-1.8); making only h inclusive (`h=s/2`) costs +3% fail sum (0.0405 vs 0.0100); no edge rule at all fails (position leaves the interval after COLDER).
