# Header count as a range array

Verified on CG (mayan-calculation perl 170, rank 1, 2026-10-22).

- `@r=1..<>` reads a header line like `4 4` (numifies to its first integer, warning only) and keeps the range: `@r` then replaces every `1..$h` list (`map~~<>,@r`, `map...,@r`)
- the same array gives the count in scalar spots: `<>/@r` (numeric context), and the width in a regex: `/.{$#r}./g` = `h` chars (`$#r` = h-1 interpolates in the pattern; h=1 gives `.{0}.`)
- beats `$h=0|<>;` + `1..$h` as soon as the range appears twice
