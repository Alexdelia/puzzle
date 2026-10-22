# Fractional action code

Encode a (cell, variant) action as one number `$k` / `$k+.5` instead of `$k.1` / `$k.3` (digit suffix):

- array index truncates: `$z[$c]` = `$z[$k]` (saves `/10`)
- `%` truncates both operands: `$c%$W` = column, `printf"%d",$c/$W` = row
- variant bit: `$c*2%2` (0 or 1), or a count `1+$c*4%4` (1 or 3: RIGHT once, LEFT = three RIGHTs); `.5` is exact in binary, `.1`/`.3` are not
- a "no-op" action can be `.5`: truthy (so `f()&&return$c` works) but indexes cell 0 (a sentinel), and `$c>1` tells it apart

In-place repeated update via alias repetition: `$_=(TABLE)[$_]for($z[$c])x$n` applies the table `$n` times to `$z[$c]` (`(LVALUE)x N` in `for` aliases each copy), 3 under `$z[$c]=(TABLE)[$z[$c]]for 1..$n`.

Verified on CG: the-fall-episode-3 perl 623 -> 612 (2026-10-22).
