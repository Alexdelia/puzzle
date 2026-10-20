# Vox Codei - Episode 2

pretty_id `vox-codei-episode-2-codesize`, interactive, 10 tests.

## Rules (as referee)

- init line `width height`; each turn `rounds bombs` then `height` rows of `.#@`. Bombs never shown in the map
- answer `WAIT` or `x y`. Strict: `x`, `8 1 3`, ` 8 1` lose with "invalid input". `x>=width` loses
- bomb on a cell showing `@` (current map) or `#` loses. Same cell twice is allowed. Placing with 0 bombs left: message only, game goes on
- turn order after the answer: place bomb (fuse 3), nodes move, fuses tick, fuse 0 explodes. Bomb placed on turn t: its blast is applied to node positions of turn t+3, and the map of turn t+3 already shows the result
- blast: own cell + 3 cells in each of 4 directions, stops at `#` and grid edge, goes through `@`, hits other bombs, which explode at once (chain)
- nodes: fixed direction U/D/L/R or none, 1 cell per turn. Next cell off grid or `#`: reverse and step back in the same turn (a node at the edge is seen there for one turn). Both sides blocked: stays. Bombs do not block nodes, nodes walk over bomb cells
- direction is hidden: read 2 maps to know it (a node at an edge facing outward behaves like facing inward)
- win: turn with empty map, the solution must still answer that turn (exit there = timeout loss)
- lose "no bombs left": after the answer, bombs 0, no armed bomb, nodes remain
- lose "maximum amount of rounds": after answering the turn with `rounds` = 1. A bomb that would clear after that turn is too late, so last useful placement is on a turn with `rounds` >= 4
- tests: rounds 30..99 (statement says < 20, wrong), test 10 is 16x12 (26 nodes, 10 bombs), test 9 "Patience" needs 3 bombs for 14 nodes

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout); statement says 100 ms, wrong

## Unverified

- order of checks for a bomb on `@` with 0 bombs left (referee: loss)
- validity of the answer on the winning turn (referee: ignored, any line wins)

## Shortest known

No short trick found. Leaderboard tops (ruby 153, perl 161, python 164) are far below anything below; their method is unknown. Verified building blocks, all on local referee, tests + validators:

- prediction of t+3 is mandatory: greedy on current map (no movement model) passes 2/20
- node state = pair (prev, cur), cells as 2D or flat index. Step: `next = 2*cur-prev` if that cell is open (in grid, not `#`), else `prev` if open, else `cur`. Covers static (prev=cur), bounce with one-turn edge stay, both sides blocked. Only `#` matters for open, `@` cells are open
- direction from 2 maps is ambiguous (test 05/07: nodes on a diagonal, two nodes meet on one cell). Reliable pairing on all 20: WAIT turns 1 and 2, for each `@` p of map0 try deltas self, R, L, D, U; accept the first q with `@` in map1 whose next step lands on `@` in map2. Then never re-read the map (except to skip cells showing `@` now): own simulation, kill nodes in the blast at placement time
- chain trap: a bomb inside the blast of a pending bomb detonates early (test 07). Fix: no bomb within 3 turns of another (cooldown or `|u-v|>=4` between planned times)
- online greedy: each turn cell with max hits at t+3, place if `hits*bombs >= alive` and cooldown over: 18/20, fails test/validator 10 (26 nodes, 10 bombs, 10 movers isolated at the end). Stricter thresholds (`>`, `+1`, `+r//k`, `*1.2`) break 01/04/06/08/09; rarity weighting (sum 1/k_i) and tie-break changes: still fail 10; restricting to cells that hit node 0: fails 09+10
- offline DFS at turn 3: precompute each path to rounds+4, target first alive node, candidates = (u, cell in blast of target at u+3, cell not occupied at u), sorted by hits desc, keep top k, recurse with bombs-1. Monotone time (u+4) or free time with gap 4: k=2..5 passes 10 but not 09 ("Patience", 14 nodes, 3 bombs, plan not found); k>=5 with free time or with pruning `hits*b>=alive` times out on 09/10 in Python (150 ms turn, planning on turn 3)
- full DFS of `solution/python/main.py` (mask dedup, conflict check) passes 20/20 at 3833 chars

Promising, untested: hybrid (greedy pass then DFS fallback), planning with all 5 direction hypotheses on turn 1 (1000 ms budget), or a faster language for wide DFS. Validators equal tests mirrored/shifted, so a heuristic passing all 10 tests likely passes validators (true for every variant above).
