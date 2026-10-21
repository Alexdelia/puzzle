# perl core via os.execute: sorted pair ends with pop

Verified 2026-10-21 network-cabling (local 21/21): `os.execute"perl -0pe'...'"` with `for$i(@y){$_-=$i-pop@y}` over sorted `@y` adds sum of top-minus-bottom pairs (foreach over an array being popped stops at the middle, odd middle gives 0).
- `sort{$a-$b}` truncates the comparator to int32: wrong for diffs >= 2^31, use `<=>`
- `/.+ /g` and `/ .+/g` (no captures) give columns as strings with a space, numeric ops ignore it
- `-$x[0]+pop@x` not `pop@x-$x[0]` (N=1: pop empties the array first)
- pure Lua is 138 for this puzzle, perl wrapper 108
