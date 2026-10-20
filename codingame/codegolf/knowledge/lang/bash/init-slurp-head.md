# `a=($(head -4))` reads the init block of an interactive puzzle

Verified on the local referee (shadows-of-the-knight-episode-2, 46/46) and CG (submit confirmed).

- referee sends the init lines and the first turn line before waiting for output, so `head -4` returns at once and consumes exactly them
- `a=($(head -4))` splits all tokens into one array: shorter than `read -a a;read;read -a p;read` (15 vs 36 bytes)
- later `read` builtin calls read byte by byte, no buffering conflict with the head
- cost: indexes shift (here W H N x y UNKNOWN = a[0..5]), print of a slice `${a[@]:3:2}`
