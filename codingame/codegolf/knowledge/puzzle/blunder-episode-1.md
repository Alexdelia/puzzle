# Blunder - Episode 1

pretty_id `blunder-episode-1-codesize`, io.

## Rules (as referee, verified against all 12 CG tests)

- line 1 `L C` (rows first), then L map rows of exactly C chars; test 12 is `30 15`, so L != C happens
- each step: if next cell in current dir is `#` (or `X` without breaker), pick first free dir from priority list, restart from list head each time (`SENW`, inverted `WNES`); then move, print dir name
- on arrival: `$` stop; `X` in breaker mode becomes blank for good; `SENW` letters set dir; `I` toggles inversion; `B` toggles breaker; `T` jumps to the other `T`, dir and breaker kept, landing `T` does not fire again
- `@` cell is blank after start
- loop: print only `LOOP`, none of the moves; exact detection = repeat of (pos, dir, inverted, breaker, destroyed-X count): destroyed count only grows, so it stands for the map
- io limit 6000 ms per test (shared probe, `knowledge/lang/_shared/io-time-limit.md`)

## Tests

- 12 CG tests, max size 30x15; only test 11 is `LOOP`
- ignoring the inverter fails tests 07, 10, 11, 12 (12 also loops if priorities stay `SENW`)
- validators 11/12 are 100x100 serpentines: 11 legit (99 moves), 12 `LOOP` (~4850 steps before the cycle); a step cap answers `LOOP` on 12 at any size, it only has to exceed the longest legit path; hidden CG validators unknown
- longest legit path: 156 moves (test 12), validators max 99. Verified: cap of 156 loop turns (check `$` at turn start) fails test 12, 999 passes all 24

## Input quirks

- final newline present on every test, no CR, no trailing spaces past C

## Shortest known

Plain Python reference 1419 chars at 100% on CG. Method below verified 24/24 (tests + validators) in Python:

- map = whole stdin as one flat mutable char array, header line kept: rows have width `w=C+1` (newline included), moves are offsets `+-1`, `+-w`; header and newlines never reached (border is `#`). Only `C` parsed (`int(t.split()[1])+1`), `L` ignored
- start `p=index('@')`; `@` never needs blanking
- direction = index `k` into an offset tuple; each turn `k = first j in (k, *priorities) whose target is not an obstacle`: current dir first, then the priority list from its head, one expression, no separate "is blocked" test
- obstacle test: target not in `'X#'[b:]`, breaker flag `b` (b=1 leaves only `#`); `B` toggles (`b^=c=='B'`)
- `X` reached (only possible in breaker mode): overwrite the cell with space. Required: dropping it fails tests 09, 12, validator 09
- `I`: reverse the priority tuple. Variant without a list: encode dirs `E0 S1 W2 N3` (offsets `1,w,-1,-w`, i.e. `1j**k` with complex coords); normal priority S,E,N,W = `j^1`, inverted W,N,E,S = `j^2` for j=0..3; keep `x` in {1,2}, `I` does `x^=3`, candidates `k,x,x^1,x^2,x^3` (C-like: loop `j^x`). Same length in Python, likely shorter where reversing an array costs
- modifier: `k=[0,1,2,3,k]['SNEW'.find(c)]`: find -1 (not a modifier) picks the last element = unchanged `k`
- `T`: `p = first('T') + last('T') - p`, no lookup table
- names: order dirs `S0 N1 E2 W3` (5-letter names first) and slice the interleaved string `'SNEWOOAEURSSTTTTHH'[k::4]` (27 chars vs 31 for `'SOUTH EAST NORTH WEST'.split()[k]`); its first 4 chars double as the modifier lookup `SNEW`. Priority S,E,N,W in this order = `0,2,1,3`
- loop: no state set, a step cap; collect output in a string, print it on reaching `$`, else `LOOP`. Python: unroll `exec("body"*999)`, body starts `'$'==m[p]>exit(print(o));` (chained compare short-circuits; exit status ignored, `knowledge/lang/_shared/crash-exit.md`), `print('LOOP')` after the exec. Cap needs 157 turns (156 moves + final `$` check); 999 used

Lengths reached (local referee, not submitted): Python 364 (exec unroll), 371 (while loop + `len(o)<9e4` cap), 372 (`x^=3` variant). Far from top Python 215 / Ruby 194: leaders use something not found here. Rough Python cost per rule: parse 60, obstacle choice 50, T 38, modifier 31, names 27, I 18, X 17, B 9.

Rejected / measured:

- ignoring X destruction: fails tests 09, 12, validator 09
- ignoring the inverter: fails 07, 10, 11, 12 (see Tests)
- breaker as blocked-set string toggled `z=z[c=='B':]or'X#'`: longer than flag `b` + `'X#'[b:]`
- per-char `exec` dispatch table for I/B/X/T: same length as four `if` lines in Python
- direction kept as char with `dict(S=w,...)` offsets: modifier shorter (`if c in P:d=c`) but dict + name lookup cost more, net about +7
- `w` from newline positions (`t.find('\n',8)-t.find('\n')`, valid since header <= 8 chars and C >= 4): longer than `int(t.split()[1])+1`
