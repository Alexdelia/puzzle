# Memo key on the cells that can still matter

Turn search over (position, grid): keying the memo on the whole grid (`"@_@z$."`) dedupes only identical grids, which barely happens on big maps. When the mover never re-enters a cell (The Fall: no upward moves, so no revisit), only its current and next cell types decide the future: `$M{"@_@z[$i>>2,S($i,1)>>2]$."}++&&return` (array slice interpolated in the string, `S` = step function).

the-fall-episode-2 perl: 6 s -> 40 ms on turn 1 of a 13x15 map, CG 100% at 652 (2026-10-22). A key with the next cell only, or with no cell, loses solutions (1 and 21 local fails of 26).

Also: `local$z[$k]=$z[$k]` (element) instead of `local@z=@z` is 4x faster on a 200-cell grid but 13 bytes longer; not needed once the key is tight. See [[memo-turn-key]].

Shorter equivalent (the-fall-episode-2 perl 594 -> CG 100% at 592, 2026-10-22): the current cell type matters only through the exit it gives, and the next cell index `$k` already encodes that exit: `S 1;$M{"$k@_$z[$k]$."}`, 4 under `"@_@z[$_>>2,$k]$."`. Put `$k` first: next cell index next to Indy's state (about 4x it) cannot be mis-split, while `"@_$k"` (last rock state glued to `$k`) can collide.
