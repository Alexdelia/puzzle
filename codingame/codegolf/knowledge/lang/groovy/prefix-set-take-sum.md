# Groovy distinct prefixes (telephone-numbers, CG 100% at 72, 2026-10-21)

`x=System.in.readLines();print((1..20).sum{x*.take(it)}.toSet().size()-1)`: `take(n)` on a short string returns it whole, `sum` of lists concatenates, `toSet` dedups; `-1` is the P9 identity (line 1 counted).
- `*.toList()*.inits().sum().toSet()` is 3 B shorter but CG scores 84%: char-list sets blow memory on the N>=100 validators (353 MB local). Avoid lists of char lists.
- `unique()` instead of `toSet()` on lists of lists miscounts.
