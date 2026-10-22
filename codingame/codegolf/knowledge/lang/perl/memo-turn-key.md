# Per-turn memo key with `$.`

Interactive search (DFS replanned every turn): `$M{"@_@z$."}++&&return` at the top of the recursive sub skips states already explored this turn. `$.` (input line number) changes every turn, so the memo resets without `%M=()` (-4). A plain global memo is wrong: states of last turn's winning path were marked, so next turn's root fails at once.

`"@_@z"` joins args and the grid array by spaces: whole state as key in 7 chars.

Verified on CG: the-fall-episode-3 perl 670 (2026-10-22), 10x faster on the slowest turn (130 ms -> 13 ms CPU).
