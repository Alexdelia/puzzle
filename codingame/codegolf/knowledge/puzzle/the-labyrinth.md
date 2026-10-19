# The Labyrinth

pretty_id `the-labyrinth-codesize`, leaderboard `the-labyrinth-codesize`, interactive.

## Rules (as referee)

- init line `R C A`, each turn `KR KC` then R lines of C chars: cumulative scanned map (`#.TC?`), not only the current 5x5
- scan: 5x5 square (radius 2, clipped at borders) around Rick at start and after every move; Rick's cell shows its own char (`.`, `T`, `C`), no Rick marker
- move: `UP DOWN LEFT RIGHT`, one cell, fuel -1 (fuel 1200). Into `#` or off map = lose. Other output = lose ("expected a movement"), no move
- first move onto C starts the alarm at A; every later move -1. Re-entering C does not reset it
- after each move, in this order: alarm started and on T = win; alarm at 0 = lose; fuel 0 = lose. So reaching T with 0 rounds left wins, and the 1200th move reaching T wins (both probed)
- T before C: nothing happens
- all 8 tests 15x30, mazes bordered by `#` except test 2 row 0 (open cells there unreachable). A in tests: 7, 21, 39, 68, 42, 33, 70, 71; test 1 A = exact distance C->T (7), shortest return needed there
- the 5x5 of every reachable cell is known in all 8 tests: fixtures complete, never-visible cells stored as `#`

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 200 ms (0.16/0.17/0.19 s pass, 0.20/0.30 s timeout); statement says 150 ms, wrong

## Input quirks

- every turn resends all R map lines: a solution must read (or skip) them, R+1 lines per turn
- statement says "Kirk" in referee lines, "Rick" in text
