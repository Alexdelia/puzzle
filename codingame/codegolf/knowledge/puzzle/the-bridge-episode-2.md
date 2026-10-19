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
