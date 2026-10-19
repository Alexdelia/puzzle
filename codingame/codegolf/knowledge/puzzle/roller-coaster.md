# Roller Coaster

pretty_id `roller-coaster`, leaderboard `roller-coaster-codesize`, io.

## Rules (as referee)

- input: `L C N`, then `N` lines one `Pi` each. output: total earnings after `C` rides
- each ride boards groups in queue order while the next group fits; a group never boards twice in one ride (sum of all groups < `L` = ride takes every group once)
- answer exceeds 2^32 (test 5: 4999975000, test 6: 89744892565569): 32-bit ints fail in typed langs
- compare: exact output line, `referee-core` io rules

## Tests (captured 2026-10-19)

- 01 `3 3 4` -> 7, 02 `10 500 1000` -> 3935, 03 `5 3 4` -> 15, 04 `10000 10 5` (sum 1500 < L) -> 15000
- 05 `100000 50000 1000` -> 4999975000, 06 `10000000 9000000 1000` -> 89744892565569
- input: final newline present, one int per line, no trailing spaces

## Timing

- io limit 6000 ms per test (shared probe `knowledge/lang/_shared/io-time-limit.md`, not re-probed here)
- naive per-ride simulation in Python times out on test 6 (C = 9e6) locally: needs a per-start (next, gain) table plus cycle skip, or similar
- per-start table is O(N^2) when sum < L (each start walks all N groups): validator 10 (N=1000, sum < L) takes 0.8 s in plain Python

## Traps

- `capture-io.sh` truncates tests 05 and 06 (CG output cap ~4 kB). Captured as zlib+b85 of stdin in 3500-char slices, one game each (journal `referee.roller-coaster.*`, scratch `capture.sh`)
- missing the "group boards at most once per ride" cap passes tests 1-3, 5, 6 but fails test 4 (97800 vs 15000) and validators 02, 03, 10
- validators cover: C=1, N=1 with L=1 and C=1e7, all groups = 1e6 with L=1e7 (answer 1e14), sum < L with huge L, ride exactly full on wrap

## Shortest known

- ours Python 905 (plain reference, 2026-10-19); top Python 106 (Zylo)
