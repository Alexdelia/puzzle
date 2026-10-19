# Power of Thor

pretty_id `power-of-thor`, leaderboard `thor-codesize`, interactive.

## Rules (as referee)

- map 40x18, init line `lightX lightY thorX thorY`, each turn input `remainingTurns` (= energy)
- move: `N NE E SE S SW W NW`, one cell, energy -1
- win: Thor on light. lose: off map, invalid move, energy 0 not on light
- test energy: 100, 13, 45, 36. Tests 2 and 4 give exactly `max(|dx|,|dy|)`: only shortest paths survive

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout); statement says 100 ms, wrong

## Shortest known

- greedy: step toward light on both axes each turn, `x+=h;y+=v`
- Python 117: `X,Y,x,y=map(int,input().split())` then `while 1:v=(Y>y)-(Y<y);h=(X>x)-(X<x);y+=v;x+=h;print(['','S','N'][v]+['','E','W'][h])`
- remainingTurns line never read in that solution: Python `input()` per turn not needed, output still accepted (CG writes it, solution ignores stdin)
