# Header array + autoflush in one guard

`$|||=@a=@F;` (11 bytes) on a `-alp` loop: `$| ||= (@a=@F)` sets `$|`
(autoflush) once and copies the 4-field init line into `@a`; later iterations
have `$|` true so `@a` is not overwritten by the per-turn line.

Costs vs four scalars (`$|||=($a,$b,$x,$y)=@F;`, 22 B): saves 11 B on the
header, but each use pays +3 (`$a[3]` 5 vs `$y` 2). It wins when two or more
fields are used at least once (e.g. phase count `$a[3]++<$a[1]` and lookup
`(W,E,N)[$a[0]%3]`).

Verified on CG 2026-10-24: power-of-thor perl 61 B scored 100% (rank 10, tie 9),
`#!perl -alp` + `$|||=@a=@F;$_=S x($a[3]++<$a[1]).(W,E,N)[$a[0]%3]`.

Note: `$| |= @a=@F` (`$||=`, no guard) re-evaluates the RHS every line and
destroys the array; the short-circuit `||=` is what makes it a guard.
