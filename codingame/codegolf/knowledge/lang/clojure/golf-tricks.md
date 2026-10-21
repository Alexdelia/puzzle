# Clojure golf tricks (blunder-episode-1, 409 bytes, CG rank 1)

All verified on local referee and CG.

- `(read)` twice reads the header numbers off `*in*`, then `(slurp *in*)` gives the rest (works inside `-main`)
- map literal as switch with default: `({\B(not b)}c b)` replaces `(if(= c\B)(not b)b)`; use one per state variable in `recur`
- `(first(remove pred coll))` beats `drop-while` (-4)
- obstacle flag as truthy value inside a lookup map: `({\X b\# 1}ch)`, toggle with `not`
- `.indexOf`/`.lastIndexOf` work on a vector of chars: `(.indexOf m\@)`
- other element of a pair: blank the current cell in a copy then index the marker again: `(.indexOf(assoc m q\.)\T)` (24 B vs 40 for `first+last-q`); bind the copy once and reuse it for the conditional update: `z(assoc m q\.)` ... `({\X z}c m)`
- string accumulator with a real newline inside the name literals, final `(println o)` (one trailing blank line is free); `(get o 999)` as step cap on a string
- loop result wrapped in `println`: `cond` branches return `o` / `"LOOP"`
- `(into()P)` reverses a vector/list (-1 vs `reverse`); `rseq` fails on a second reverse
- `\$o` reads as one char token: needs `\$ o`
- `\#1` too: put the number first or a space
