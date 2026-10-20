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

## Shortest known (algorithm, verified on local referee 2026-10-20, 25/25)

Method: offline DFS over bomb cells at turn 1, then replay the plan.

- grid as one flat array, rows joined by `#`, plus `#`*(w+1) at the end: stride `W=w+1`, `x=i%W`, `y=i//W`. Row separators stop horizontal blasts; the trailing pad stops downward blasts and, through negative-index wrap (Python, Ruby, Perl), upward blasts from row 0. Languages without negative indexing need a pad row on top too (or bounds checks)
- blast of cell `i`: `i` plus, per direction `d` in `1,-1,W,-W`, up to 3 steps while cell is not `#`. Score = count of `@` in it
- `f(g,b)`: if no `@` left: success. Else scan cells in index order (row-major); take a cell only if it is `.` in the current DFS grid and `b*score >= nodes left` (each bomb kills at least the average still needed). Copy grid, set every blast cell to `.`, recurse with `b-1`; first success wins
- destroyed nodes become `.` at once in the DFS. On replay, a bomb whose cell was `@` in the ORIGINAL grid gets 2 `WAIT` lines before it: the node is gone 3 turns after the bomb that killed it, and 2 waits always suffice (that bomb was at least 1 turn earlier). Needed: t09, v02, v08, v09, v13 plans use such a cell
- timing ignored by the DFS: every case fits (worst v13: 9 bombs + 2 waits + 3 fuse = 14 <= 19 rounds)
- after the plan, 3 more lines are needed (bomb explodes 3 turns later, win checked after the action of that turn). Repeating the last placement line 3 times works: stacking on an armed bomb is accepted, a cell freed by its own explosion is `.`, and with 0 bombs a well-formed `x y` is a wait. `WAIT` x3 works too
- `b` comes from the first turn line (second number); the remaining turn lines can be left unread
- referee: printing the whole plan at once at turn 1 then blocking on input (`while 1:input()`) passes locally. Exiting after printing fails locally (`solution closed its input` / `solution exited`): the process must stay alive. Printing ahead on CG: unverified, safe fallback = one line per turn with a read in between

Why short: no simulation of fuses or chains, no rounds bookkeeping, one recursive function with one prune.
Needs: recursion (or explicit stack), array copy, integer division/modulo. No regex, eval, big ints.
Cost (Python 3.11 locally): v13 (19x19, 9 bombs) 518 DFS calls, 0.6 s for the first turn, limit 1 s: close in Python, keep the `.` test before scoring and do not compute the blast twice if it matters. t10/v10 ~640 calls, 0.2 s. Everything else no backtracking.
Lengths: readable Python 476 bytes (not golfed, scratch only). Leaderboard tops (ruby 135, perl 153, python 160) are far below a golfed DFS estimate: likely a simpler heuristic or hardcode tuned to the hidden CG validators, which local validators (generated, plus v12-v14) reject (unverified).

Rejected (with reason):

- online greedy max hits, pending nodes marked: 20/25; with "place only if `score*bombs >= left`" or "bombs>1 or score>=left": 22/25. t11/v11 trap: centre cell hits 4 of 8 nodes with 4 bombs, then 4 isolated nodes left for 3 bombs; t10 also fails
- online first-fit in scan order with `score*bombs >= left` (no backtrack): 21/25 (t10, t11, v11, v13)
- DFS with prune `score>0` only: does not finish in 60 s on the suite
- DFS scanning cells in reverse index order: v13 needs 5000 calls, 6 s
- DFS with `score*b > left`: wrong (1 bomb 1 node never accepted), runs away
- 2 `WAIT`s before every bomb (no original-grid test): t03 8 bombs in 15 rounds does not fit
- score as `count('@')` before minus after applying the blast on a copy (shorter, no separate score loop): Python v13 6.4 s, timeout on v13 and flaky on t10/v10 (22-24/25). Fine only where the language is ~10x faster than CPython; measure v13 first-turn time before relying on it

## Bash/perl DFS size (2026-10-20)

- perl DFS (flat string grid, `y/@//` counts, `substr` lvalue blast, `1while<>` after `$|=1`) inside `perl -e'...'`: 25/25 local, 426 bytes, so the exact DFS is ~2.4x the bash top 177
