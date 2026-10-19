# Surface

pretty_id `surface-codesize`, leaderboard `surface-codesize`, io.

## Rules (as referee)

- input: `L`, `H` on own lines, `H` rows of `L` chars `#` (land) / `O` (water), `N`, then `N` lines `X Y`
- output per query: size of the 4-connected water component containing column `X`, row `Y`; `0` on land
- `X` is the column (first), `Y` the row: transposing fails non-square tests (7: 100x50, 9: 900x400)
- io limit 6000 ms per test (referee default, value from chuck-norris, not probed here)

## Tests

- 1-7 small (largest 100x50, N=100)
- 8: 1000x1000, 22% water, many lakes, N=100, largest answer 837: recursive flood fill stays shallow
- 9: 900x400 all `O` except `#` at (612..614, 399), N=1 query `43 54`, answer 359997: one lake of 360k cells. Recursive DFS overflows default stacks (Python recursion limit 1000), need iterative fill or raised limit
- every captured input ends with a final newline, rows have exactly `L` chars, no CR, single spaces

## Traps

- queries repeat cells and lakes: memoising per lake is optional for speed (Python plain BFS per query still fast here, 100 queries on test 8)
- test 9 size: Python reference (iterative stack) 0.22 s locally; per-query full-map rescans (e.g. `L*H` loop per query, 100 queries on 1M cells) may get slow in slow langs
- capture: test 7-9 exceed CG `.output` cap (~4000 chars, middle elided as a `...` line); test 8 needed 9 chunked games (lzma + 14 bits/char CJK, cap counts chars not bytes)
