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

## Shortest known

Method (verified 23/23 local tests + validators, Python, 2026-10-20), re-planned from scratch every turn, state = grid only (apply own rotation to stored grid):

1. Rock kill first: for each rock, walk its path with current grid (same move rule as Indy, no rotation). At every cell on it that is unlocked (`t>0`), try RIGHT then LEFT; accept the first rotation whose new type rejects the rock's entry side AND after which the Indy DFS below still succeeds with zero budget for the first cell. Output it
2. Else Indy DFS from Indy's cell, entry, budget 1: step to next cell (fail off grid / not accepting), success on entering `(EX,H-1)`. Next cell options: as is (cost 0), R (1), RR (2), L (1); locked only as is; need `budget>=cost`, recurse with `budget-cost+1`. Output the first rotation on the found path (RR outputs RIGHT, the next turn's DFS asks for the second one), WAIT if none

Why it works: rotations on cells 1..k of the path need total <= k (one per turn, rotation before the move); doing the earliest needed first is EDF, optimal. The rock kill spends a turn only when the DFS proves the plan survives it.

Needed (each probed by dropping it, local referee):

- DFS over all paths to the exit: depth-limited (2..8 cells) fails 6-7 cases (dead ends on the bottom row near the exit)
- exit read: success "Indy leaves the grid downward" instead of `(EX,H-1)` fails 7/23 (other bottom cells exit down)
- double rotation option: without RR 7 fails
- dedupe rotated types per cell (2/3, 4/5 two-cycles, 0/1 fixed give duplicate branches): without it Python times out at 150 ms on tests 04, val 04, 12. A compiled lang may not need it (unverified)
- rock kill must walk the whole rock path, not only the rock's next cell (test 07 rows are locked except one cell): next-cell only fails 2
- kill must make the cell reject the rock (else it rotates the same cell forever): fails 3
- kill must try both directions: RIGHT only fails 2
- kill must be verified by the zero-budget DFS: unverified kills fail 8 (steal turns Indy needs). Kill only when the DFS says WAIT fails test 07 (DFS asks for its earliest rotation even with slack, so it never WAITs)
- NOT needed: excluding Indy's cell / rock cells from kills (0 fails), stopping at the first rotatable cell of a rock path (0 fails either way)

Encodings that shorten it (all in the 23/23 Python):

- move table as one 42-char string `T[type*3+entry]`, entry T=0 L=1 R=2, value = new entry (0 down, 1 east, 2 west, 3 blocked): `333000312033230103312030300003233133330303`; `dx = d and 3-2*d`, `dy = d<1`
- RIGHT as tuple `0,1,3,2,5,4,7,8,9,6,11,12,13,10`; LEFT = RIGHT applied 3 times
- bounds free: append a 0 to each row and a row of 0s (x=-1 wraps to the pad in Python), type 0 rejects everything
- input entry `'TLR'.find(s[0])`

Lengths reached: Python 1051 (semi-golfed, passes 23/23, not the cell's job). Leaderboard tops are 178 (ruby) to 455: 1051 is far, so either heavy golf of this method or a different, much simpler method exists. Unknown: no shortcut found here. Hints for cell workers (unverified):

- python 201 = ruby 178 + ~23 wrapper bytes (`import os;os.system("ruby -e'...'")`): if ruby runs in the CG python box, the ruby core is the real target for many langs (see `knowledge/lang/_shared/system-wrapper.md`)
- referee quirks on CG not probed: whether rotating Indy/rock cells or bad commands are fatal (our referee: fatal)
