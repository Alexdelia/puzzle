# Negative-index wrap for a cyclic walk of exactly one lap

Verified on the local referee and CG (roller-coaster perl 139 -> 135, 2026-10-22).

- cyclic walk over `@p` (length `n`) from `i` for at most one lap: run `$j` from `$i-$n` up to `$i`. Negative `$j` reads `$p[$n+$j]`, non-negative `$j` reads `$p[$j]`, so `$p[$j]` is always `p[(i+k) mod n]`: no `%$n` inside the walk, lap bound is `$j<$i` instead of `$j<$i+$n`
- the next start is `$j%$n` (Perl `%` with positive right side is non-negative)
- works in any language with Python-style negative indexing (Python, Ruby, Perl); not in JS arrays
- boolean conjunction `A<B&C>=D` with bitwise `&`: 1 under `&&`, `&` binds looser than relational ops and the booleans are numeric; no short-circuit, so the right side must be safe to evaluate (here `$p[$i]` exists)
- `($l,$c,$n,@p,$_)=@F` under `-ap`: the slurpy array takes all, `$_` gets undef, so the answer accumulates in `$_+=` and `-p` prints it; switch the loop to `while$c--` (same length as `for 1..$c`) so `$_` is not aliased
