# `for$t(@p)` aliases one coordinate per phase

Verified (shadows-of-the-knight-episode-2, 281 -> 246 with the related tricks).

- `for$t(@p){...$t=$q;print"@p\n"...}`: assigning `$t` edits the printed pair, no index variable
- per-phase size via `$h=($S=shift@S)-1` (`@S` from `$|=@S=<>=~/\d+/g`)
- response test without `ord`: `$_=<>;/S/?A:/W/==$q*2>$s?B:C` (`/W/` is 1 or "", `==` binds looser than `>`)
- `$q-$q%$S` is true iff `$q` is outside `0..S-1` (perl `%` with positive right operand is non-negative)
