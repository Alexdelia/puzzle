# Don't Panic

pretty_id `don't-panic` (slug `dont-panic`), interactive. Same game as The Paranoid Android episode 1.

## Rules (as referee)

- init line `nbFloors width nbRounds exitFloor exitPos nbTotalClones 0 nbElevators`, then `nbElevators` lines `floor pos`; each turn `cloneFloor clonePos LEFT|RIGHT` of leading (earliest unblocked) clone, `-1 -1 NONE` if none
- generator position is not in the input: it is the clone position of turn 1 (always floor 0, RIGHT)
- clones spawn at turns 1, 4, 7, ... at generator, heading RIGHT, LEFT if a blocked clone sits on the generator
- turn t: BLOCK blocks leading clone (BLOCK on NONE: no-op, no error). Win if a clone is at exit in the turn-t state, even when that clone is blocked that same turn
- move: clone on elevator cell -> floor+1, same pos, direction reversed if a blocked clone is there. Else next cell blocked -> reverse, then step one in new direction (11 RIGHT, blocked at 12 -> next turn 10 LEFT). Off map -> destroyed
- lose: turn nbRounds without win ("Max rounds count reached"), or all clones spawned and none unblocked alive ("no more clones")
- win on turn exactly nbRounds counts (test 7: 54 turns, nbRounds 54, greedy reference has zero slack)
- tests 6 and 7 one elevator per floor, all clones 3 cells apart: greedy rule `BLOCK iff target (elevator or exit of clone floor) is behind the clone` passes every test
- unverified: stacked elevators (same pos on floor f and f+1) assumed to chain one floor per turn; output other than exact `WAIT`/`BLOCK` assumed to lose

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.13 s pass, 0.14 s timeout once at turn 5, 0.16 s timeout); statement says 100 ms, wrong

## Input quirks

- NONE turns happen when every clone out is blocked or destroyed and the next is not out yet; floor `-1` must not crash a lookup keyed by floor
- every line ends with newline; fields single-space separated
