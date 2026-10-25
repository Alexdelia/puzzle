# Parity-biased reflection bisection: drop the equality case

Verified on CG (shadows-of-the-knight-episode-2 perl 224 -> 223 -> 200, 2026-10-24, rank 1 100%).

In a 1D bisection where you jump to `q=l+h-o` (so the reply compares the bomb with the interval midpoint `(l+h)/2`) and you must special-case the equal reply (SAME: set BOTH bounds), force the threshold to a half-integer instead:

- jump `q=($l+$h|1)-($o=$t)`: `+` binds tighter than `|`, so `$l+$h|1` = `($l+$h)|1`; then `s=o+q` is odd (unless the out-of-range edge rule fires)
- an odd `s` means the threshold `s/2` is not an integer, so an integer bomb can never reply equal, and `q==o` (2o=s odd) is impossible: both the equality branch and the midpoint fixed-point fix disappear
- update one bound to `($s+$x)>>1` where `$x=/W/^($t<$o)` (1 = raise l, 0 = lower h); `>>1` truncates the half-integer threshold exactly
- pick the target with a symbolic reference instead of a ternary: `$x=/W/^$t<$o,${$x?l:h}=$s+$x>>1` (barewords l/h are the variable names, no strict subs; the comma is legal in a for third clause)
- after the edge-rule fix `s` may be even: then both branches yield `s>>1` = the threshold, still safe; only the equal-reply early exit is lost

Saves ~23 B in a bisection loop (SAME branch + fixed-point fix out, symbolic-ref single assignment in), costs ~1-3 extra jumps per axis.

## Fold the parity fix into the out-of-range rule (Ruby verified, CW all languages)

Do not fix q after the edge formula; compute the edge base first and set q's low bit directly:

- edge base `b = t + S` if the reflection q0 is in range-positive, `b = t` if q0 < 0; e.g. Ruby `b=t+S-S*q[63]` (`Integer#[]` sign bit)
- then `q = (b>>1) | 1 ^ (t&1)` (`|`/`^` same precedence, left to right): the low bit becomes `(t+1)&1`, so `t+q` is odd in the edge branch too
- the normal branch `q=(l+h|1)-t` already has odd `t+q`; only the edge branch needs it
- saves the separate `q^=~t-q&1` / `q^=...` line (Ruby 217 -> 215, CG 100%); same trick works in any language with xor/and/shift and a sign test (`q<0`, `q>>31`, `q>>63`)
