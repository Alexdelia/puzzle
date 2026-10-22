# Chained comparison

Perl 5.32+ (CG runs 5.34) chains relational operators: `$z[$_]>1>$h{$_}` = `$z[$_]>1&&1>$h{$_}`, 3 under `$z[$_]>1&&!$h{$_}` (undef counts as 0). Middle operand evaluated once.

Verified on CG: the-fall-episode-3 perl 670 (2026-10-22) uses it in a candidate filter.

Two upper bounds on one value: `$S>=$_-$r<=$l` = `$_-$r<=$l&$_-$r<=$S` (min cap without `min`), 7 under the `&` form. Verified on CG: roller-coaster perl 115/105 (2026-10-22).

Two different bounds on two values: scale the second so both share the middle constant. `$d<3&&9>$e` (e in {0,1,2,8,9}) = `$d<3>$e/3` (9/3 = 3 fails, 8/3 passes), -1. Short-circuits like `&&`. Verified on CG: the-fall-episode-3 perl 584 (2026-10-22).
