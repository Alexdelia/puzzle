# Clear `$_` for free under `-ap`, build output in `$_`

Verified on the local referee (mayan-calculation bash perl-wrapper, 2026-10-20, CG accepted 193).

- under `-p` the answer is printed from `$_` at the end, but `$_` still holds line 1
- with `-a`, `($x,$y,$_)=@F` reads the two header fields and sets `$_` to undef in the same statement (one more target than `@F` has fields): `-a` costs 1, `=@F` is 3 shorter than `=split`, and no `$_=""` needed
- then prepend output with `s/^/X/` (1 shorter than `$_=X.$_`), works on undef `$_`
- `for`/`map` alias and restore `$_`, so loops in between do not disturb the cleared value
