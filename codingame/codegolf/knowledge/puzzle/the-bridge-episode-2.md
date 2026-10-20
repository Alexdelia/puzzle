# The Bridge (episode 2)

pretty_id `the-bridge-episode-2-codesize`, leaderboard `the-bridge-episode-2-codesize`, interactive.

## Rules (as referee)

- init: `M`, `V`, then 4 road lines (`.` safe, `0` hole), all lanes same length `L` (30..65 in tests). Full road given, no hidden state
- each turn: `S`, then M lines `X Y A` (dead bikes keep a line, `A`=0)
- all bikes start at X=0, same speed; tests 1..12 start speed 0..8
- order per turn: speed change (`SPEED` +1, `SLOW` -1 floored at 0), then move by new speed
- `WAIT`/`SPEED`/`SLOW` and blocked `UP`/`DOWN`: dies on first hole in x+1..x+S, same lane
- `JUMP`: dies only if landing cell x+S is a hole
- `UP`/`DOWN`: checks current lane x+1..x+S-1 and new lane x+1..x+S (current-lane cell x+S not checked). Speed 0: lane changes, nothing checked
- rails: if any ALIVE bike would leave lanes 0..3, nobody changes lane, move is straight. Dead bikes never block
- lose: alive < V (checked after the move). Win: alive bike X > L after the move (X == L is not enough, game asks another turn)
- turn limit: the 50th output always loses, even if it would carry the bikes past L. So cross with at most 49 outputs
- cells beyond L: no holes
- invalid command: lose (referee assumption, not probed)

## Dead bike line (rarely matters)

- straight: X = first hole hit; jump: X = landing cell; Y unchanged
- UP/DOWN: per step x+i, current lane checked before new lane; current-lane hole keeps old Y, new-lane hole reports new Y
- unverified: whether all current-lane cells are checked before any new-lane cell (no test layout separates it)

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout), matches statement

## Input quirks

- road lines have no trailing spaces; bikes listed in lane order of start (Y ascending)
- test 7: M=3 on lanes 0,1,2; test 10: M=2 on lanes 0,2; test 12: M=2 on lanes 1,2 (lanes not contiguous from 0)
- a plan computed on turn 1, then one command per turn while reading (and ignoring) each turn input, passes on CG. Skipping the per-turn reads: unverified

## Reference

- plain Python DFS over (x, speed, alive lanes, turn), memo of failed states, plan once on turn 1: 1841 bytes, 100% on CG

## Shortest known (algorithm search 2026-10-20, local referee, 23/23 tests+validators)

Method: greedy with fixed-depth lookahead, replanned every turn from the turn input.

- each turn try commands in order `SPEED, UP, JUMP, DOWN, SLOW`; output the first one after which a 4-ply sequence (this move + 3 more, same order) keeps `>= V` bikes alive at every ply. No win test, no turn counter, no memo
- other orders that pass: `SPEED,DOWN,JUMP,UP,SLOW`, `SPEED,UP,DOWN,JUMP,SLOW`. Only SPEED-first orders pass
- depth: 3 plies fails 2 validators (05, 07); 4..7 plies all pass. Cost 5^4 = 625 nodes per turn, far under 150 ms even in Python
- WAIT never needed (JUMP dominates it: same move, only the landing cell checked). All 4 other commands needed: no 4-command subset passes at any depth up to 7 plies
- per bike: survives iff no `0` in `lane[y][a:e] + lane[y+d][a:e+1]`, `e = x + new speed`, `a = x+1` (straight, UP, DOWN) or `a = e` (JUMP). Same expression covers all 5 commands (d=0 for straight, current-lane slice empty for JUMP, both empty at speed <= 0)
- rails: append a 5th lane of holes (`'0'*99`): `lane[4]` and `lane[-1]` (Python) both hit it, so a rail-blocked move just kills that bike in the model. Real referee blocks the move instead; still 23/23 because replanning each turn absorbs the mismatch
- SLOW needs no clamp at 0: negative speed in the lookahead only gives empty slices; greedy never relies on it
- turn state: x = max X over all bikes (dead bike X <= alive X), alive lanes = Y where A=1. In Python: `B=[[*map(int,I().split())]...]`, `max(B)[0]`, `[y for x,y,a in B if a]`
- command index trick (order above, i in 0..4): speed delta `(i<1)-(i>3)`, lane delta `i%2*(i-2)`, jump `i==2`, name `'SPEED UP JUMP DOWN SLOW'.split()[i]`
- needs: string slicing (or char indexing loop), recursion or nested loops of depth 4, no big ints, no regex

Lengths: Python 385 B (vs top 188: big gap, either heavy golf or a shorter method not found here; treat as open)

Rejected:
- full DFS to the far side (plan once, print all lines at once): exponential, test 12 timeout at 1000 ms in Python with order SPEED,JUMP,UP,DOWN,SLOW; no order of any 3..6-command subset solves all 23 cases within 20000 nodes each; also needs win test + turn limit (SLOW to negative speed recurses forever)
- shallow greedy + SPEED cap (SPEED only while speed < c, c 1..15, all 120 orders): 1 ply best 10 cases fail, 2 plies best 6 fail
- simpler lane-change check (only new lane, both lanes full x+1..e, only old lane): fails 4..9 cases at 3..6 plies
- printing the whole plan on turn 1 and exiting: passes locally, unverified on CG
