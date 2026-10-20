# Alias input for interactive golf

`I=input` once, then `I()` for every read: with 6 reads it saves 16 bytes (shadows-of-the-knight-episode-2: 294 -> 278). Verified on the local referee.

Related, same puzzle: `z=input().split()` + `int(z[i])` beats `[*map(int,...)]` for a list used once per axis; `p+S*(q>0)>>1` needs no parentheses (`+` binds tighter than `>>`); `(a)^(b)` is one byte shorter than `(a)==(b)` when the branches can be swapped.
