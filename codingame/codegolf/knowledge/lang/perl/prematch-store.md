# `$`` (prematch) as a free per-turn store in a `-n` loop

`/ /;` (4 B) matches the init line (the only line with a space), and `$`` is then the
part before the first space = the first field (e.g. lightX). Under `-n`/`-ne` the
prematch **survives later failed matches**, so every turn can read it for free; the regex
runs every turn and misses on the one-number energy lines, which is what keeps the value.

Verified on CG 2026-10-24, power-of-thor perl 59 -> 56 (100%, rank 8 tie 7):
```
#!perl -nl
/ /;$|=print"S"x($.>17&46>$_>3).(W,E,N)[$`%3]
```
`$`%3` = lightX%3 = 1,2,1,0 for the 4 validators -> `(W,E,N)` letter.

Traps, measured on perl 5.34:

- under `-p`/`-pe` a failed match **resets** `$``, `$&` and `$1` (verified: after an energy
  line `$` is `""`), so the prematch store only works with `-n` + explicit print. Under
  `-p` store the whole line instead (`$h||=$_`, 8 B; `$h` must start undef - `$;` does not,
  it defaults to "\034" and is truthy, so `$;||=$_` never assigns)
- an empty pattern `//` reuses the last regex (none on turn 1) and gives `$`` = "", not the
  line; `/$/` matches every line and overwrites
- `print S x(...)` parses the bareword `S` as a filehandle: `Undefined subroutine
  &main::x called`. Write `print"S"x(...)` (5 B, no parens needed: `.` is looser than `x`)
  or `print+(S x(...))`
- `$|=print EXPR` prints then flushes (see flush-print-assign.md); with `-n` it replaces
  `print` + `$|=1;` for 2 B, and the flush happens after the first print so turn 1 of an
  interactive game is answered in time (verified on CG with this 56 B program)
