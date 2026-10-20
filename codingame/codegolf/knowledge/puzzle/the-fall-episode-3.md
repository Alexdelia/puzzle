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

## Shortest known

General method (passes local 4 tests + 7 validators, python 869 B, 2026-10-20). Tops (python 117, perl/ruby 122, bash 129, js/php 139) are far below any solver: almost surely hardcoded answers keyed on the hidden CG validators (unverified, same pattern as `dwarfs-standing-on-the-shoulders-of-giants`, `mars-lander-episode-3`). Local validators are synthetic (mirrored + spawn-shifted), so `submit.sh` refuses such hardcodes; probing hidden validators by submit is the user's decision. The general method gives a first submission, not rank 1.

Method: replan every turn with a plain DFS over turns, return the first action of the first plan that reaches the exit.

- state: grid, Indy (cell, entry side), visible rocks. Future spawns are unknown: replanning each turn plus candidate order below is enough on all 11 local cases
- candidates per turn, in this order: rotate RIGHT then LEFT each of: Indy's next cell, the cell after it, each rock's next cell; WAIT last. Skip cells with type <= 1 (covers locked negatives, 0, 1), Indy's cell, rock cells, exit cell. Duplicates harmless
- candidate cells come from the path WITHOUT checking that the cell accepts the entry (the cell to fix does not accept yet)
- simulate: rotate, Indy step (must have opening on entry side), win if Indy enters exit cell (before any rock check), rocks that share a cell in the current state vanish without moving, other rocks step (gone if no opening), Indy loses if any moved rock is in his new cell, recurse
- table per (type, entry side): exit direction, or "opening without exit" (type 4 from LEFT, 5 from RIGHT, 6 from TOP), or "no opening". The middle state matters: a stuck rock stays one turn and crushes Indy entering it (test 04 turn 24). Encode exit direction d so the new entry side equals d: 0 = down (enter TOP), 1 = right (enter LEFT), 2 = left (enter RIGHT); entry side from input word = `len(word)-3`
- no memo and no depth limit needed (Indy never makes a U-turn, so depth <= cells). Python without memo: worst game 60 ms local total; a memo (set of str(state)) makes it ~3x faster on 01 if a hidden case is slow
- golf shapes that help: flat grid with a 0 sentinel column (stride W+1) and a 0 padding row, so left/right/down moves are `k-1`, `k+1`, `k+W` with no bounds check; rotation by a 28-char lookup string (RIGHT row + LEFT row, `ord-48`), exit table as one 42-char digit string indexed `abs(t)*3+side`; `"RLIEGFHTT"[D::2]` gives RIGHT/LEFT
- language features: recursion, list copy (or rotate/undo in place), no regex/eval/bigints

Rejected (each fails local suite):

- WAIT first in candidate order: 02 + mirror no plan, 04 crushed by an unseen later rock
- RIGHT only (no LEFT): 03, validators 02/07 no plan
- only Indy's path as candidates (no rock cells): 01/02 + mirrors (rocks must be blocked by rotating their next cell)
- only Indy's next cell: fails 01/02 (double rotations need 2 cells ahead)
- dropping rocks that share a cell after moving instead of before: 02 turn 7 (two rocks meeting above Indy's next cell)
- stuck rock dropped at once: 04 turn 24
