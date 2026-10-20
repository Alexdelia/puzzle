# `-E` is refused on the `#!` line

`#!perl -anE` dies with `Can't emulate -E on #! line` (verified locally, perl 5.42). Use `-l` + `print`, or `-M5.010` for `say`. Verified 2026-10-20. Native Perl dont-panic: `#!perl -anl` loop, hash slice `@e{@F[0,3]}=@F[1,4]` (undef key `""` harmless, array slice would clobber `$e[0]`).
