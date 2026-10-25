# `-F` split: word chars become fields, non-word chars become empty fields

Verified 2026-10-25 locally on CG Perl 5.34 (ascii-art hidden referee).

`perl -lF'\w\K|.'` (shebang form `#!perl -lF\w\K|.`, 17 B incl newline) splits each
chomped line so that

- every word char becomes its own field,
- every non-word char is consumed by `.` and yields an **empty** field,
- a line whose chars are **all** non-word chars gives `@F = ()` (`scalar(@F) == 0`):
  split drops empty fields when every char is a separator (same for
  `split/./,"####"` / `split/./,"ab"` -> 0 fields).

Measurements (`-l` chomps before the split):

- `"AB!!CD"` -> `A B "" "" C D` (6 fields)
- `"#### ####"` (ascii-art font row) -> 0 fields
- an ascii-art font-5 row containing `_` -> hundreds of empty-string fields with `$F[0] eq ""`

Then `map-1+ord()%32,@F` is a whole per-char glyph-index map: a letter gives
`ord%32-1` (0..25, both cases), an empty field gives `-1` (`ord("") == 0`), `_`
gives 30 (out of range for a 27-glyph slice -> prints nothing).

Cost: the shebang pattern is +6 B (`\w\K|.`), the map expression drops the `*/\w/`
test (-4 B: `map-1+ord()%32,@F` = 17 vs `map-1+ord()%32*/\w/,@F` = 21), net **+2 B**
against `#!perl -lF` (empty pattern = chars in `@F`).

Boolean dispatch: `@F` is false on all-non-word lines; `$F[0]` is false on lines whose
first field is empty (font rows containing `_`). Both are usable as a line test but
`$F[0]?` (6 B) and `@F?` (3 B) lose to `$.<4?` (5 B) / `$.>3?` (5 B) once the shebang
cost is counted.

Also measured: `print+()` under `-l` emits a newline, so a ternary that prints on the
three header lines cannot be silenced with an empty list.

Related: [char-split-F](char-split-F.md) (same trick without a pattern: `-F` empty),
[numify-skip-line](numify-skip-line.md), [shebang-no-E](shebang-no-E.md).
