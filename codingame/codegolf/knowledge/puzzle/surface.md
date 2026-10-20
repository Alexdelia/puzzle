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

## Shortest known

- ours Python 177 (local, 2026-10-19, not submitted), top 154 (Wazemo)

Method (verified 9/9 tests + 8/8 validators local, Python 3.11, 2026-10-19):

- one flat char array of the raw rows WITH their `\n` (Python: `L,H,*r=open(0)` keeps the newlines, `''.join(r)`), stride `w=L+1`, cell `(X,Y)` at `Y*w+X`, neighbours `i±1`, `i±w`. The `\n` ends are the left/right walls, no column bound check
- the lines after the rows (`N`, queries: digits, spaces, `\n`) contain no `O`: joining them with the rows is harmless. Only the down-overflow of the last row can run past the end (test 9: `N=1`, tail 8 chars < `L=900`): pad at least `w` non-`O` chars (Python `+L*w`: the `L` line string repeated, 4 chars). Up-overflow of row 0 gives negative indexes: wraps to the pad in Python; langs with nil/undefined on out of range may need no pad at all (unverified)
- query index without parsing ints: line `"X Y\n"` with `' '` replaced by `'+w*'`, then eval (`eval(q.replace(' ','+w*'))`)
- flood = growing work list iterated while appended (`for i in k:` + `k+=i-1,i+1,i-w,i+w` when `g[i]` is `O`): BFS without a separate queue/pop, no recursion
- no grid copy per query, no counter: mark each visited water cell with the query's work list object itself (`g[i]=k`). `len(k) = 1+4*size`, so answer = `len(g[start])//4`: fresh lake -> its own `k`; lake flooded by an earlier query -> that query's `k`; land `'#'` -> `len 1 //4 = 0`. Needs a mutable array that can hold a reference (list), and a length that maps land to 0
- fallbacks if marking by reference is long in a lang: copy grid per query + counter (Python 178, `s=g[:]`), or mark with `0` and `s.count(0)` (Python 172 but 4.9 s on test 8: the 1M-cell count per query is too slow, rejected)

Constraints that shape the method:

- test 9 / validator 07: single lake of 360k / 300k cells. Recursive flood dies: Python recursion limit; C recursion with gcc -O0 segfaults locally under the 8 MB default stack (CG stack size unverified); JS/Ruby default limits likely too small (unverified). Perl recursion is heap-based (unverified here, no perl in this shell)
- test 8: 1M cells, 100 queries: per-query O(L*H) work (full copy, full count) costs ~10-50 ms each in Python; a plain list copy passes (0.8 s), `count` + rebuild does not reliably
- answers need no memo for speed (repeat queries are rare on big lakes), the `g[i]=k` mark is about length, not speed

Rejected (Python): token split + set of `O` indices with per-query set copy (230); set-frontier flood on complex keys (building the set alone is ~65 chars); dict `pop(i,0)` mark (needs `dict(enumerate(...))`, +13); unconditional one-line mark `k+=[...]*('O'==g[i]);g[i]=k` (marks land beside the lake, a later land query reads the lake size); slicing queries as `r[int(H):]` (the `N` line evals to an index and prints an extra line)

Perl 158 (local 17/17, CG rank 3, 2026-10-20), `-ln0`: strip header + `$w=L+1` in one `s`, `@g=split//`, `while(/(.+) (.+)/g)` over queries, `for(@{$k=[$s=$1+$2*$w]})` growing list, mark `$g[$_]=$k` when `$g[$_]=~O`, `print@{$g[$s]}>>2` (land '#' = symbolic empty array = 0, no pad needed: out of range cells are undef). Rejected: regex-spread flood on the string (`o(.{L})\KO|O(?=.{L}o)`, lookbehind capped at 255 so use `\K`; needs /s): correct but times out on 360k lake and 1M grid.
