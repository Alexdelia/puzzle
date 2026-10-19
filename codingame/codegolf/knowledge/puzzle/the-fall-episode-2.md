# The Fall - Episode 2

pretty_id `the-fall-episode-2-codesize`, leaderboard `the-fall-episode-2-codesize`, interactive. 7 CG tests, 6-7 with rocks.

## Rules (as referee)

- init: `W H`, H rows of W signed types (negative = locked), `EX`. Exit cell = (EX, H-1), never rotatable
- turn input: `XI YI POSI`, `R`, R lines `XR YR POSR`. Indy always starts entering from TOP
- action: `WAIT` or `X Y LEFT|RIGHT` (LEFT = counterclockwise). Rotation applied before the move of the same turn
- rotations (RIGHT): 2<->3, 4<->5, 6->7->8->9->6, 10->11->12->13->10, 0 and 1 fixed
- move: exit side from current cell type + entry; crash if next cell off grid or does not accept that entry side
- win: Indy enters the exit cell (no need to leave it). Test 1 wins on turn 2
- rotating a locked cell = loss: "Invalid instruction: you cannot rotate cell (2 0) as it is locked" (verified). Rotating Indy's cell, a rock's cell or an out-of-grid cell = loss in our referee (unverified on CG)
- rocks: pre-scheduled (turn of first appearance, cell, entry), same moves as Indy, crash = gone from the next input already. Input lists active rocks in fixed definition order, not spawn order (test 7: follower rock `8 0 TOP` spawned turn 2 listed after rocks spawned turns 3-6)
- tests 6-7: one rock per turn from (9,y) RIGHT for y=2..6, turns 2-6; test 7 adds a rock entering Indy's start cell at turn 2, one step behind Indy. Rock schedule assumed independent of player actions (unverified)
- unverified in referee: Indy vs crashing rock in its old cell counts as hit; swap (Indy and rock crossing) is not a hit; rock-rock meetings ignored

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout), as statement

## Input

- grid rows single-space separated, no trailing space. Rock lines follow the R line, R=0 gives no line
- referee transcript matches CG input byte for byte on all 7 tests with the reference solver
