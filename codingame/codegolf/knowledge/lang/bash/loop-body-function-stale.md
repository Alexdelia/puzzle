# Function holding print+read+update

Verified (shadows-of-the-knight-episode-2).

- `e(){ echo ...;read r;((update));}` called from the loop and once after it: the extra update after the last jump runs on stale variables, harmless when the bounds are reset by the next phase or the game ends
- saves the duplicated echo/read/update text
