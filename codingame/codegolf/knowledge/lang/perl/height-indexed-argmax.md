# Height-indexed array argmax

Small-int keys (0..9) -> argmax without sort: store index at slot key, last slot = argmax.

- `map$a[<>]=$_,0..7` reads 8 lines in order, `$a["9\n"]` numifies to slot 9; `$a[-1]` = index of max (last max on ties)
- stale slots from earlier rounds must go: `@a=print...` resets to `(1)`, slot 0 only, harmless while some key > 0
- `$|=map...` as the `while` condition: map count 8 is true, autoflush for free
- the-descent: 42 vs 46 of `chop(@a=sort map<>.$_,0..7)` (`chop-list-argmax.md`), verified local 15/15 + CG 100% (2026-10-22)
- same idea in any language with auto-growing arrays (JS `a[h]=i`, Lua, PHP): no max/sort/index call, slot reset per round
