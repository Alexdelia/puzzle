# Vox Codei - Episode 1

pretty_id `vox-codei-episode-1`, leaderboard `vox-codei-episode-1-codesize`, interactive.

## Rules (as referee)

- init: `width height`, then `height` rows of `.@#`. Each turn: `rounds bombs` (rounds counts down, first turn = test value)
- output per turn: `WAIT` or `x y` (x column, y row)
- turn order: 1) armed bombs fuse -1, fuse 0 explodes, chain explosions same turn; 2) action; 3) win if no `@` left; 4) lose if bombs 0 and no armed bomb; 5) lose if rounds was 1
- bomb placed in turn t explodes in turn t+3 (placed at rounds r, explodes at rounds r-3; rounds 4 placement still wins in last turn)
- blast: own cell + 3 cells each of 4 directions, stops at `#` and border, passes through `@` (destroys it) and bombs; any armed bomb hit explodes same turn
- explosions run before placement: bomb placed in a cell hit this turn is not triggered; may place on cell of a `@` destroyed this turn
- stacking a bomb on a cell holding an armed bomb is accepted (consumes a bomb)
- tests: 11, rounds 4..15, bombs 1..9, grids up to 15x12; test 9 needs placing on a node cell freed by an earlier bomb

## Output parsing (probed on CG 2026-10-19)

- `WAIT` case-insensitive (`wait` ok); `a` fails
- placement must be `digits space digits` exactly: leading zeros ok (`00 01`), `0 -1` fails, trailing space fails
- x/y out of grid fails; bomb on `@` or `#` fails
- with 0 bombs left any well-formed `x y` is a wait, even out of grid or on a node (golf: no need to print WAIT after last bomb, unverified for malformed lines: assumed fail)

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: ~140-150 ms (0.135 pass, 0.14 once fail once pass, 0.16 fail); statement says 100 ms. Referee uses 140 ms
