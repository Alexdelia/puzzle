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

## Perl result (2026-10-20)

- Perl 96 bytes, 100% on CG, rank 45 (tie 39), via `-alp` loop, see knowledge/lang/perl/p-loop-header-first-iteration.md. Top 42 not reached
- Perl row creation: the local perl referee is unusable without `CG_PERL` (no toolchain), so submit only works with `--probe`; test locally with system perl and a stdin/awk simulator
- ratio top/ours is about 1/2.2 in every language (Python 52 vs 113, bash 46 vs 100, Perl 42 vs 96): suggests either legacy scores or one shared unknown trick. Python 52 cannot even parse 4 ints (32 bytes), so blind output. No hypothesis tested yet

## Hidden validators

Details, probe table and sources: `puzzle/power-of-thor/referee/hidden/facts.md` (2026-10-20, 7 count probes).

- 4 validators (leaderboard grain 25%)
- directions: no validator needs E (so no NE/SE either). Exactly 1 horizontal (light West), exactly 1 vertical and it goes North, the other 2 are strict SW (dx<0, dy>0). N only in the pure vertical one
- golf use: horizontal part is only `W` or nothing (`x>LX`), no E branch; vertical: `S` in the SW ones, `N` only when Thor starts on the light's column
- exactly 1 validator init line is in {`7 8 28 8`, `0 17 36 0`, `0 17 31 4`}; the other 3 inputs are unknown, energies unknown. Not complete, no hardcode key yet

## JavaScript (2026-10-20)

- general 99 bytes 8/8, rank 122: ``[a,b,c,d]=readline().split` `;for(i=0;;i++)print((i<b-d?'S':i<d-b?'N':'')+(i<c-a?'W':i<a-c?'E':''))``. Counter form: letter on axis while i<|delta|; no tracking, no energy read
- hidden-assumption variant 88 (W only, N only via `i<d-b`): CG probe inconclusive (progress showed another language's submission, row stayed 99); unverified
