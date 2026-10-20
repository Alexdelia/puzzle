# Response word evaluated as an arithmetic variable

Verified (shadows-of-the-knight-episode-2).

- `read r` gives `WARMER`/`COLDER`/`SAME`; inside `(( ))` `r` expands to the variable of that name: unset = 0
- `COLDER=1` once, then `r` is 0 (WARMER) or 1 (COLDER) in arithmetic: `r^q>t` is `r^(q>t)` (relational binds tighter than `^`), replaces `[[ $r = W* ]]` + `$?`
- SAME separated by length: `${#r}<5` (SAME is 4, the others 6)
- ternary else branch with assignment needs parens, middle branch does not: `c?l=1:(h=2)`
- `h=--s/2` saves two bytes over `h=(s-1)/2` when s is dead afterwards
