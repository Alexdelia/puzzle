# glob brace expansion as a string table

`<"...">` (csh_glob) expands `{a,b}` braces like a shell and returns words without `* ? [` unchanged, no filesystem check. Double quotes inside the `<>` keep spaces in one word, so a table whose entries share affixes compresses into one pattern; nested braces expand outer-first, left to right (cartesian order).

- `(<"O({1,log n,n{,^2,^3}{, log n}})">)[i]` = `O(1)`, `O(log n)`, `O(n)`, `O(n log n)`, `O(n^2)`, `O(n^2 log n)`, `O(n^3)`, `O(n^3 log n)`: 9 under `(1,$l="log n",map{$_,"$_ $l"}n,"n^2","n^3")` even before moving `O(`..`)` into the pattern (saves 2 more: `print+(...)[i]//"O(2^n)"` vs `print"O(",(...)[i]//"2^n",")"`)
- unquoted spaces split into separate patterns; `\ ` inside `<>` is eaten by the double-quote interpolation (needs `\\ `); quoting the whole pattern is cheapest
- `<n^{2,3}>` = `"n^2","n^3"` minus 2 (`^` and `(` `)` are not glob-magic)
- `$var` interpolates inside `<"...">` before expansion

Verified: blunder-episode-3 perl 110, local 26/26, CG-accepted 2026-10-22 (submit.sh).
