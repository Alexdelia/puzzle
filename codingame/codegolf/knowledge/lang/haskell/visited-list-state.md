# Haskell: state as visited-position list, `last(x:[y|c])` as if

Verified (blunder-episode-1, 24/24, CG 399):
- `>>=\s` lexes `=\` as one operator: write `>>= \s`; `g'T'` lexes as identifier `g'`: write `g 'T'`
- mutable grid not needed: keep list of visited positions, head = current; "destroyed X" = X in the list (`elem t h`); saves ~25 bytes over take/drop rewrite
- `last(x:[y|cond])` is `if cond then y else x`, 4 bytes shorter than the `if`
- pattern guards `|(q,r,c):_<-[(j,t,c)|...]=` replace `let{..}in` and `head`
- cap loop: `case splitAt 998$f.. of(a,[])->unlines a;_->"LOOP"` on a lazy list
- `interact$ \s->` shorter than `getContents>>=`
