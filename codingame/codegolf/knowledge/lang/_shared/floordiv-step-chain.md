# Threshold chain as floor-division steps

`T//i` is 1 while `i` lies in (T/2, T] and 0 once `i > T`. So a piecewise-constant schedule over a counter `i`

    i<T1 ? R1 : i<T2 ? R2 : 0

equals `(T1-1)//i*(R1-R2) + (T2-1)//i*R2` as long as every `i` used is above `(T2-1)/2` (else a term becomes 2). No conditionals, no `and/or` spacing.

- verified (mars-lander-episode-3, Lua 2026-10-21): `169//i*100-184//i*89` replaces `i<170 and 11 or i<185 and-89 or 0` (20 vs 33 chars, Lua 5.4 needs a space between a numeral and `or`/`and`). Referee + CG accepted
- trap: if the counter starts too low, shift it (`io.read()*5+99`, thresholds +99) instead of letting `T2//i` reach 2
- integer counter needed for integer output (`169//i` on a float prints `1.0` in Lua, Python)
- languages: Lua/Python `//`, Ruby `/` on Integers, Perl needs `int`, C-family `/` on ints (truncates, same for positive operands)
