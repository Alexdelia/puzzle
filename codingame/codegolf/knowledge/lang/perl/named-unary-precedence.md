# named unary ops bind looser than arithmetic

`ord`, `chr`, `length`, `lc`, ... (named unary operators) sit below `* / % x + - . << >>` in perlop, above `< > & | ^`.

- `ord$2%25` is `ord($2%25)`, `ord$2>>9` is `ord($2>>9)`: wrap the operand, `ord($2)%25`
- `ord$1&15` is `(ord$1)&15`: bit ops and comparisons after a bare named unary are safe, no parens
- `ord($1)*ord($2)%25` needs both parens; `ord$1^ord$2` needs none

Verified locally with CG perl 5.34 (vox-codei-episode-2, 2026-10-22).
