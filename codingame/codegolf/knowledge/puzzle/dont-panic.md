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

## Shortest known

Method (verified, all 15 local cases, CG test 7 zero slack): greedy, one map `floor -> target`, target = elevator pos, exit pos on exit floor. Per turn `f p d`: BLOCK iff target strictly behind the clone, i.e. `(t-p)*s<0` with `s=+1` RIGHT, `-1` LEFT. Everything else WAIT.

Why short:

- uniform line loop, no elevator count: read init line for exit (fields 4,5), then every line `f p [d]`: no `d` -> store `e[f]=p` (elevator line), `d` present -> print. Init line must not print (one extra output line shifts all actions one turn late: fails 13/15). `nbFloors width nbRounds nbClones nbAdditional nbElevators` all ignorable
- set-if-unset alternative: `e[f]||=p` on every line is safe (every clone floor already has a target), only `f=-1` gets junk; careful when the lang treats `0`/`"0"` as false (exit pos 0 exists, validator 03)
- direction from one char or length: `d>"M"` (NONE also > M, harmless), `ord(d[0])-79` (R +3, L -3), `2*len(d)-9` (RIGHT 5, LEFT 4). Spaceship form `(p<=>t)==(d<=>"M")` also right (0 at target never equals +-1)
- NONE turns (`-1 -1 NONE`) occur 12 times over the cases with this policy: lookup of floor `-1` must not crash, output on NONE is free (BLOCK on NONE is a no-op)
- exit floor = top floor, no elevator there, one elevator on every lower floor (all cases)

Cross-language: rank 1 lengths match a `system()` wrapper around a short shell core (C 72 = `main(){system("");}` 19 + bash 53; Java 159 fits a ProcessBuilder wrapper + 53). Python `import os;os.system('''bash -c '...' ''')` running the bash core won CG test 7 (verified 2026-10-19). See `knowledge/lang/_shared/system-wrapper.md`. Every non-bash cell should compare its native solution with wrapper + shortest core.

Lengths reached: bash 103 (CG accepted 2026-10-20, rank 28; tricks in `knowledge/lang/bash/` read-in-order, missing-command-print, word-as-arith-var; 102 with `for((;;))` instead of while, untested on CG). Earlier: bash core 121 bytes (2 lines, ungolfed, CG verified via Python wrapper); python native uniform loop ~150 (not golfed). Top bash 53 implies a much shorter core than 121: unknown, cell workers to find (ideas: `${e[f]=$p}` set-if-unset, `read` of the init line inside the loop, arithmetic on `${#d}`).

Rejected (local referee, python):

- `(t-p)*s<=0` (block at target too): 3/15, blocked clone on elevator kills it
- `(p>t)!=(d<"M")` xor: 5/15, wrong when `p==t` LEFT (happens 56 times)
- block only when RIGHT past target: 7/15
- one-turn-late actions (extra init print): 2/15
- output other than exact WAIT/BLOCK: CG answers `invalid input. Expected 'WAIT, BLOCK' but found ''` for an empty line (verified test 2)
