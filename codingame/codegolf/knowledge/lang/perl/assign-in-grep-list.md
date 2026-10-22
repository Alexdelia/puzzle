# Assignment as the list of grep/map

`grep{BLOCK}@e=<>` instead of `@e=<>;...grep{BLOCK}@e`: the list assignment runs first and returns `@e`, so the block already sees the full `@e` (e.g. `"@e"=~/.../` against all lines). Saves 3 (`;` and one `@e`) when the block needs the whole array.

Verified on CG 2026-10-22: dwarfs perl 38 -> 35, `print 2+grep{/ /;"@e"=~/ $`<LF>/}@e=<>` scored 100%.
