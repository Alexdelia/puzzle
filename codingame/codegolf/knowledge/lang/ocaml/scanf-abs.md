# Read one int: `Scanf.scanf" %d"abs`

Verified locally and on CG (roller-coaster row 257, 2026-10-22).

- `Scanf.scanf" %d"abs` returns the int (positive inputs): 1 byte shorter than `Scanf.scanf" %d"(+)0`, 6 shorter than `(fun x->x)`. `(+)0` works too: `scanf fmt (+)` returns `(+) x`, then applied to `0`
- for signed input use `(+)0` (`abs` drops the sign)
- `Array.init n(fun _->Scanf.scanf" %d"abs)` reads `n` ints, one per line or space separated (init runs in index order)
- first format of the program needs no leading space when input starts with a digit: `Scanf.scanf"%d %d %d"(fun l c n->...)`
- `1to`, `1then`: "Invalid literal" (the letter is read as a literal suffix), keep the space after a digit
