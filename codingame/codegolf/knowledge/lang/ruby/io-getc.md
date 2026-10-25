# `getc` does not exist at top level; `$<.getc` reads one char

Verified Ruby 3.1.2 (`CG_RUBY`) 2026-10-25: a bare `getc` is a NameError
("undefined local variable or method 'getc'"). The one-character reader is
`IO#getc`:

- `$<.getc` (7 B) reads one byte from stdin (ARGF) and returns a 1-char String
  (nil at EOF); `STDIN.getc` (10 B) is the explicit form.
- `$<.getc.hex` (11 B) is the cheapest first-char hex read; `.ord`/`.sum` are
  the same length. `Kernel#gets` does exist, so `gets.sum` (8 B) stays the
  cheapest whole-line byte sum, and `gets.hex`/`gets.oct` (8 B) the cheapest
  leading-number parses.

Do not use `getc.hex`-based keys: they only work after `$<.` and are longer
than `gets.*` keys. (A 2026-10-25 power-of-thor journal note claiming
`getc.hex` as an 8 B key was wrong; the candidates failed with NameError.)
