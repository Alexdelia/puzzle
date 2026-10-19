# Shadows of the Knight - Episode 1

pretty_id `shadows-of-the-knight-episode-1`, leaderboard `shadows-of-the-knight-episode-1-codesize`, interactive.

## Rules (as referee)

- init lines: `W H`, `N` (jumps), `X0 Y0`. Each turn one line: direction of the bomb from Batman, vertical letter first: `U D L R UR UL DR DL`
- answer `X Y`: Batman jumps there (any window, not only adjacent), jumps left -1
- win: Batman on bomb window after the jump (checked before the jump count: winning on jump N is a win)
- lose: N jumps used, not on bomb ("you are too late"); x or y out of `0..W` / `0..H` ("invalid input. Expected '0 <= x < 4'")
- jumping to the current window is allowed, costs a jump
- bomb fixed during a game, never printed by CG logs (only read off the win frame); same bomb in every log seen, no evasive behavior despite test names "Evasive", "Not there?"
- tests: 4x8 N40, 25x33 N49, 40x60 N32, 1x80 N6, 50x50 N6, 100x100 N7, 9999x9999 N14. Tests 4-7 need near-optimal bisection (binary search with `(lo+hi)//2` uses exactly N on 4-7)

## Output format (probed on CG 2026-10-19)

- `0 2 ` (trailing space): accepted
- ` 3  5` (leading space, double space): fail "Expected 'x y'"
- `3 5 9` (extra token): fail "Expected 'x y'"
- referee model: trim end, split on first single space, both halves must parse as integers

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s and 0.99 s pass, 1.05 s timeout)
- next turns: 150 ms (0.135 s and 0.145 s pass, 0.16 s timeout); statement says 150 ms, right

## Traps

- direction with no horizontal letter means bomb column = current column: a bisection must collapse that axis to the current value, not keep the old range
- the `N` line is never needed by a correct bisection: read and discard
