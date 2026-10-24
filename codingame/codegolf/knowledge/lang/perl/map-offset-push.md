# map offset push: block-form work list, 1 B under enumerating `$k+`

In a growing work-list flood with the block form `for$k(@k){...}` the four grid
neighbours are normally pushed as

    push@k,$k-1,$k+1,$k-$w,$k+$w        # 21 B + push@k,

`map` builds the same list from constant offsets in one byte less:

    push@k,map$k+$_,-1,1,-$w,$w         # 20 B + push@k,

`map` aliases `$_` to each offset, but `$k` is the loop alias, so the offsets are
safe. This only works in the block form: with `... for@k` the current position is
`$_` itself and `map` would clobber it (use `map$_+$_,...` is then wrong).

Verified on the surface referee (5.42 locally, perl semantics): replace-only
tests 02/05/09 pass with 121 B vs 122 B, same output.

Costs:

- about 1.5-2x slower per flooded cell than the plain four-argument `push`
  (map builds a 4-element list per cell): surface IDE test 09 (360k-cell lake)
  1.0 s -> 2.0 s, synthetic 1M-lake validator 4.5 s -> 6.7 s, i.e. over the
  6000 ms local bar. Only use when the time margin is large.
- `$_` is unusable inside the map block, so the grid must live in a named string
  (block form) - it cannot be combined with `for@k` flooding of `$_`.

Trap found with the same flood: `y'X'O'` (or `s/X/O/g`) after 4-arg
`substr($t,$k,1,X)` marks counts **land as well as water** - the 4-arg substr
replaces every tested cell, and boundary land cells are tested. A land query then
prints a positive count. To get an exact transliteration count, mark only water
with `substr($t,$k,1)=~s/O/X/` (23 B, +3). Verified on the surface referee
(land-only test 02 printed 1 instead of 0).
