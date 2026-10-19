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

Method (generic, verified 8/8 local): greedy, each turn step toward light on both axes, `v=sign(LY-y)`, `h=sign(LX-x)`, `y+=v;x+=h`, print `N/S` part then `E/W` part. Always a shortest path (diagonal first, then straight), so exact-energy tests 2, 4 and validators 1-4 pass.

- never read the per-turn energy line: print in an infinite loop, CG feeds stdin, solution ignores it; extra lines after the win ignored
- one sign per axis (`(a>b)-(a<b)`, `<=>`, ternary), no abs/max/distance
- direction letter without branch: `v*v*'SN'[v<0]` (Python 13 chars, vs `['','S','N'][v]` 14); operand order matters, `'SN'[v<0]*v*v` is wrong (`'N'*-1*-1` = `''`)
- negative index on 3-list `('',S,N)[v]` works where lists index from the end (Python, Perl, Ruby)
- Python: method + letter trick reached 113 locally (8/8), was 117 with list indexing
- C ternary-with-comma form (`y<T?T--,"N":y>T?T++,"S":""`) used in our C 123 / C++ 139

Rejected / refuted (CG probes 2026-10-19, games 910142761, 910142788):

- several moves in one line: CG answers `Expected a movement (one of N, NE, E, SE, S, SW, W, NW) but found 'SSSSSSSSSSSSS'`, lose. Output must be exactly one direction per line
- off-map clamp: `SW` forever on test 4 dies at (18,18) `Thor wandered off the path and died (invalid position)`. No clamp
- constant direction per axis (no tracking): fails validator 4 (horizontal must stop) or test 3 (vertical must stop)
- straight-first driven by energy line (`V if E<=|dy|`): no tracking, but fails slack tests 1 (E 100 vs 21) and 3 (45 vs 31)
- precomputed strings `'S'*(dy)+'N'*(-dy)` then chop/slice per line: no sign needed, but longer in Python (120) and Perl (100 vs 92 cmp form, untested locally: no perl toolchain)

Open: leaderboard tops (Ruby/Perl 42, bash 46, Python 52, C 65) are below what parsing 4 ints + loop + 2 tracked axes costs (Ruby parse alone 26: `a,b,c,d=eval gets.tr' ',?,`, unverified). Unexplained: either legacy scores from an older test set / counting rule, or an unknown trick. Do not assume the tops are reachable with the generic method.
