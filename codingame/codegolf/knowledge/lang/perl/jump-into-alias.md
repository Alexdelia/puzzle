# Compute the new value straight into a foreach alias, keep the old one inline

Verified on CG (shadows-of-the-knight-episode-2 perl 246 -> 232, 2026-10-22).

When a loop moves a position held in an aliased variable (`for$t(@p)`, printed as `"@p"`) and later needs both old and new values, do not compute into a temp `$q` and copy back (`$t=$q`). Assign the new value into `$t` and save the old one where `$t` is first read:

- `$t=EXPR_USING($l+$h-($o=$t))` : `($o=$t)` captures the old value in the expression that needs it
- tests become `$t>$o` (old vs new) instead of `$q*2>$s` with the sum
- out-of-range fix inside the test: `$t=FIX if($t=$l+$h-($o=$t))-$t%$S` (assignment in the left operand, re-read on the right: the left operand is the `$t` SV, so both sides see the new value)

Trap: Perl pushes variables as SVs, not values: `$t+($t=5)` with `$t=1` gives 10, not 6. `0+$t+($t=5)` gives 6 (the `0+` makes a temp). So `$s=$t+($t=...)` does not keep the old value.

Also: when a sum `$s` is known even (equality case of a bisection), `$s/2` is 1 under `$s>>1`.
