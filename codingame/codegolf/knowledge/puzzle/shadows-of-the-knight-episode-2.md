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
