# The Fall - Episode 3

pretty_id `the-fall-episode-3`, leaderboard `the-fall-episode-3-codesize`, interactive.

## Rules (as referee)

- init: `W H`, H rows of W ints (single spaces, negative = locked), `EX`. Exit cell = (EX, H-1)
- each turn: `XI YI POSI`, `R`, R lines `XR YR POSR`. Turn 1 shows Indy on his start cell (y=0, TOP)
- output `WAIT` or `X Y LEFT|RIGHT` (LEFT = counterclockwise). RIGHT cycles: 2<->3, 4<->5, 6->7->8->9->6, 10->11->12->13->10; 1 unchanged
- turn order: rotate, then Indy moves one room, then rocks move, then new rocks spawn (shown in that turn's input)
- Indy moving into a room with no opening on the entry side: "crashed on a wall". Indy in a room with no path from his entry (type 4 from LEFT, type 5 from RIGHT, type 6 from TOP): "stopped". Both lose on the move
- win: Indy enters the exit cell (same move, before any rock check)
- rocks: fixed spawn schedule per test (turn, cell, side; from an edge, may spawn on Indy's start cell behind him). Move like Indy. Wall crash / stuck / leaving the grid (also down through the exit cell): removed at once, absent from next input
- collisions are checked lazily: Indy and a rock in the same cell after a move are both shown in the next input, death ("crushed") comes after that turn's answer. Two rocks in one cell: both shown once, removed before the next move
- rock list order: fixed per-rock order of the level, not spawn order (fixtures keep CG order)

## Tests

- all 4 tests have rocks (statement: "last two tests only", wrong)
- 01: 11x10, 5 rocks, rock-rock collision. 02: 6x6, 4 rocks. 03: 14x8, 2 rocks. 04: 14x14, 5 rocks, 38 turns
- pure WAIT dies on every test; tests 1/2 need a rotation on turn 1 (a solver that WAITs while the path is still fixable later loses to rocks)
- validators: mirrored tests (x -> W-1-x, 4<->5 7<->9 10<->11 12<->13, LEFT<->RIGHT) + spawn turns shifted

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout)

## Unverified

- rotating a locked/0/Indy/rock/exit cell or a malformed command: referee loses with an error; CG behavior not probed (could be ignored there)
- swap (Indy and rock exchanging cells in one move) counts as no collision in the referee; not observed on CG
