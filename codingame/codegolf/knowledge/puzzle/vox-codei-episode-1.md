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

## Shortest known (algorithm, CG-verified 2026-10-20)

Best method: offline random-restart search at turn 1, then replay the plan. Python 548 (ungolfed-ish) = 100% on CG, rank 15 at that time.

- `#` never changes, so the blast of a cell is static: precompute per cell the SET of node indices its blast hits (own cell + up to 3 steps per direction, stop at `#`). Search state is just the set `r` of live nodes; no grid copy, no recursion
- flat grid: rows joined by `#`, plus `#`*(w+1) pad at the end: stride `W=w+1`, `x=i%W`, `y=i//W`; upward from row 0 wraps (negative index) into the pad and stops. Char order `#` < `.` < `@` gives one-compare tests (`c<'.'` wall, `c>'.'` node). Only compute blasts for non-`#` cells (a pad cell would step past the end)
- restart: `r` = all nodes, `b` = bombs; draw ~999 random cells; accept cell `i` if its blast set is non-empty, `i` not in `r` (empty now, or a node already killed in the plan) and `len(blast&r)*b >= len(r) > 0` (the `>0` also stops at b=0 and after success). On accept: `r -= blast`, `b -= 1`, append plan line. Restart until `r` empty
- plan line for a cell that was `@` in the original grid gets 2 `WAIT` before it (node gone 3 turns after the bomb that killed it). Tail of 3 `WAIT`. Print whole plan at turn 1, then `while 1:input()` (CG-verified: printing ahead and blocking passes)
- bombs from first turn line: `int(input()[-1])` (bombs < 10)
- local: 10/10 suite runs 25/25 with the static-set version (python, worst case v10 well under 1 s). The earlier version recomputing blasts on a list copy each draw timed out v10 in 1 of 9 runs

Why short: no fuse/chain simulation, no rounds bookkeeping, no recursion, one prune. Needs: sets (or bitmask ints: blast as int mask, `popcount(m&r)`, `r&=~m`), random. No regex, eval, big ints (bitmask needs > 64 bits on 19x19: big ints or a set type).

Previous best: offline DFS in scan order with the same prune (still valid, 25/25 local, perl 426); random search drops the recursion and grid copies.

Leaderboard tops (ruby 135, perl 153, python 160) are still far below: unknown method, see Hidden validators.

Rejected (with reason):

- online random choice among cells with `s*b>=left` (pending-aware): 20-23/25 in 6 runs, t10/v10 fail every run: no lucky-resubmit path

- online greedy max hits, pending nodes marked: 20/25; with "place only if `score*bombs >= left`" or "bombs>1 or score>=left": 22/25. t11/v11 trap: centre cell hits 4 of 8 nodes with 4 bombs, then 4 isolated nodes left for 3 bombs; t10 also fails
- online first-fit in scan order with `score*bombs >= left` (no backtrack): 21/25 (t10, t11, v11, v13)
- DFS with prune `score>0` only: does not finish in 60 s on the suite
- DFS scanning cells in reverse index order: v13 needs 5000 calls, 6 s
- DFS with `score*b > left`: wrong (1 bomb 1 node never accepted), runs away
- 2 `WAIT`s before every bomb (no original-grid test): t03 8 bombs in 15 rounds does not fit
- score as `count('@')` before minus after applying the blast on a copy (shorter, no separate score loop): Python v13 6.4 s, timeout on v13 and flaky on t10/v10 (22-24/25). Fine only where the language is ~10x faster than CPython; measure v13 first-turn time before relying on it

## Bash/perl DFS size (2026-10-20)

- perl DFS (flat string grid, `y/@//` counts, `substr` lvalue blast, `1while<>` after `$|=1`) inside `perl -e'...'`: 25/25 local, 426 bytes, so the exact DFS is ~2.4x the bash top 177

## Hidden validators

Facts (CG probes 2026-10-20, python, our row was 2471):

- probe 1: online greedy max-hit, killed nodes timed as pending (cell reusable 3 turns after the killing bomb, WAIT while the best cell is pending), first max in scan order. Local 22/25 (fails t11, v10, v11): NOT 100% on CG. So at least one hidden validator defeats plain greedy (a t10/t11-like trap). Greedy-only golf is ruled out
- probe 2: random-restart search above: 100% on CG (python 548 committed)
- `/progress` `score` is the best submission's, so a probe is 1 bit (100% or not); failing counts are not visible

Guesses (unverified):

- tops 135-160 may use a randomized ONLINE heuristic (random cell among those passing `s*b>=left`) that fails with some probability and was resubmitted until lucky; or a hardcode keyed on grid size for the trap validators with greedy elsewhere. Neither tested
- local validators v10/v11 are mirrored/shifted copies of t10/t11 made by our referee task, not the real ones

Probes spent: 2 of 10 (task puzzle.vox-codei-episode-1.-.1792516443190153296).

