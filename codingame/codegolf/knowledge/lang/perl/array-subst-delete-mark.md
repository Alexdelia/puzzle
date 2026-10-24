# `$a[$i]=~s/O//` = test-and-delete a grid cell (array grids)

On an array of chars, `$a[$i]=~s/O//` returns 1 iff the element was `O` and then
empties it, so it is the whole flood condition ("was water, now visited") in 13 B.
The string form `substr($t,$i,1,X)=~O` is 20 B (4-arg substr marks and returns the
old char). Out-of-range indices are undef and negative indices wrap, but neither is
fatal and the tail has no `O`, so no end pad is needed (unlike 4-arg substr, which
dies past the string end).

Costs that usually kill it: the char array must be rebuilt per query for repeat-lake
correctness. `@k=...;@u=split//;$u[$_]=~s/O//&&push@k,...` = 122-123 B on surface,
but splitting a 1M-char input per query is ~60 ms: surface IDE test 8 (1M, 100
queries) times out locally (6 s) and hidden validator 8 has 300 queries. A persistent
array + per-query `@u=@F` copy is shorter to build once (`@F=split//`, 11 B) but is
128 B and still copies 1M SVs per query.

Verified locally + surface referee (8/9, test 8 timeout), 2026-10-24. Useful where
the grid is small or queried once.
