# Chained comparison

Perl 5.32+ (CG runs 5.34) chains relational operators: `$z[$_]>1>$h{$_}` = `$z[$_]>1&&1>$h{$_}`, 3 under `$z[$_]>1&&!$h{$_}` (undef counts as 0). Middle operand evaluated once.

Verified on CG: the-fall-episode-3 perl 670 (2026-10-22) uses it in a candidate filter.

Two upper bounds on one value: `$S>=$_-$r<=$l` = `$_-$r<=$l&$_-$r<=$S` (min cap without `min`), 7 under the `&` form. Verified on CG: roller-coaster perl 115/105 (2026-10-22).
