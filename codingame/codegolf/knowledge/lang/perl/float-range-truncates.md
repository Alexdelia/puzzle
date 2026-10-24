# float range endpoints truncate to int

Verified 2026-10-24 with CG Perl 5.34 (`join",",...`):

- `2.5..4` -> `2,3,4` (start truncated toward zero, then +1 steps)
- `-1.5..1` -> `-1,0,1`
- `0..1.5` -> `0,1` (end truncated)
- `3..3.9` -> `3`
- `4..2.5` -> empty (so `$#a/2..$#a` works as `int($#a/2)..$#a`, no `int`)

Useful for slice half-splits like `@a[0..$#a/2]` without `int`.
