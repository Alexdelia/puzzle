# Turn-counter modular window (phase) instead of a two-sided energy window

Per-turn interactive hardcode where the plan differs per hidden validator only by a phase
(count of moves of one token). A cheap phase from the turn counter `$.` and a header value V:

```
V > -$. % 2V        # 9 B; V = a per-class header number (e.g. lightY)
```

Perl `%` takes the sign of the right operand, so `-k % 2V` = `2V-k` for k<=2V-1, wraps to 0 at
2V, then `2V-1`, `2V-2`, ... So the predicate is true for `k = 2V-V+1 .. 2V` (V truths... in
general k in (2V-V, 2V) = (V,2V)) and false again after the wrap. A validator whose game ends
before the wrap keeps only `2V-V-1 = V-1`... concretely with V=17: k=18..34 true (17), and a
shorter validator ending at k=31 gets 14. Two validators needing different phase counts (17 vs
14) share one 9 B expression; a linear threshold `V>2V-$.` (8 B) gives the longer one 19 and
cannot trim the tail.

Verified on CG (power-of-thor perl 51 B, 100%, rank 5, 2026-10-25): plan V3 needs 14 diagonals
in its 31 turns, V4 17 in its 36, V1/V2 none; `$'>-$.%34` passes hidden 4/4.

Notes
- needs a header-derived per-class constant V and a turn counter; both must be cheap to read
  (`$'`/`$`` come from a 4 B `/ /` match, `$.` is free).
- the same form works in Ruby/Python (floor-mod on negatives gives the positive divisor's
  sign); NOT in C/JS/Go (truncated mod gives a negative value: use `-$k%m` differently).
- the phase is a comparison (0/1), so `"S"x(P)` prints exactly one `S`; a bare arithmetic
  expression would repeat `S` count times.
