# Plan-once DFS: print at the leaf, never exit

Verified on CG (the-bridge-episode-2 perl 287, 2026-10-22, 100%; was 303 with `$|=` + `sleep print$q`).

- when CG accepts extra output lines after the game ends (interactive, whole plan printed on turn 1), the leaf can just `print` and let the DFS run on: no `exit`, no `sleep`, no `$|=`. The search keeps finding sibling plans, the 8 kB block buffer fills and flushes within ms (all 23 local cases < 15 ms to the first byte). Only safe when every leaf below the first found one is quick to reach (here: past the bridge every move survives)
- guard at sub entry: `sub f{@_<$V||98<length$q&&print($q)||map{...;f(...)}-2..2}`. TRAP: `print$q||map` parses the map as print's argument, write `print($q)`
- outer loop as `map{...}-2..2` instead of `for$j(-2..2){...}` with `$_` as the loop index, -6 B; `local` inside a map block is restored per iteration like in a `for` body. Needs everything that uses the index computed outside the inner `map` over `@_` (inner map rebinds `$_`): e.g. a per-command length `$l=$_&&$s-1` hoisted out of the per-bike map
