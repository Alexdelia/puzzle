# `y///` with an empty replacement list counts without changing the string

`y/CHARS//` (empty replacement list) is the identity translation: it returns the number of translated characters and leaves `$_` (or the target) unchanged. A count test with no side effect, shorter than a `/.../` match plus `$&` handling when the target has no other copy of the chars.

```perl
$_="aCa"; $n=y/C//;      # $_ still "aCa", $n == 1
```

- `$n=y/C//` is 8 B vs `$n=()=/(C)/g` or `$n=tr/C//`.
- With `/d` an unpaired search char is deleted instead (`y/ab/X/d` = a->X, b deleted), so `//` and `//d` differ.
- Verified on CG perl 5.34 (`$CG_PERL/bin/perl -w`), the-labyrinth 2026-10-24.
- Trap: warnings are on for some spellings; the empty list itself is silent.
