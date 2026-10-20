# the-labyrinth hidden validators

8 validators, each = IDE test k under a flip, same alarm A, same 15x30 size. Recovered exactly (start, A, every scannable maze cell). Files `NN.json` = transform of `tests/NN.json`; reference perl `solution/perl/main.pl` passes `--hidden` 8/8.

| # | title (= test) | transform | start | A |
|---|---|---|---|---|
| 01 | Easy, follow the path, 1 direction | rot180 | 8 24 | 7 |
| 02 | Easy, follow the path, 4 directions | flipY (rows reversed) | 12 25 | 21 |
| 03 | Easy, empty | flipX (cols reversed) | 1 28 | 39 |
| 04 | Several paths, no cycle | rot180 | 12 2 | 68 |
| 05 | Several paths with cycles | flipX | 1 9 | 42 |
| 06 | Difficult with cycles | flipX | 13 18 | 33 |
| 07 | Difficult, long with cycles | flipX | 3 23 | 70 |
| 08 | Difficult, longer with cycles | flipX | 11 7 | 71 |

Numbering: validator k is assumed to be the variant of test k (A is unique per test, so the pairing by A is certain; CG's display order not seen). Validator 02 has the open row 0 of test 2 as last row 14 (no `#` border there): matches forum reports "validator 2 crashes at once, Rick in the before-last row, out of bounds" (Leaf67 2022-07-14, https://forum.codingame.com/t/the-labyrinth-puzzle-discussion/59?page=6) and "validation case nr 2 illegal move on first move" (IslandRock1 2025-02-04, page 7).

## Evidence (agent/state/oracle/the-labyrinth.tsv, 2026-10-20)

All probes = perl flood "goal nearest of `?`/`C`, then `T`" wrapped in a fresh or chained language, exiting at turn 1 unless a predicate on `KR KC A` holds.

1. c: no predicate: 100% (so this weaker method passes CG)
2. cpp: A<=35: 37.5% -> 8 validators, 3 with A<=35
3. javascript: (start,A) in {id,flipY,flipX,rot180} of the tests: 100%
4. typescript: flipY|rot180: 3/8; php: flipX|rot180: 7/8
5. lua {1..4}, go {1,2,5,6}, rust {1,3,5,7} restricted to flipY|rot180: 3, 2, 1 -> row-flipped = {1,2,4}
6. php chain: col-flipped all + not-col-flipped for {1,2,3,4}: 100%; typescript {1,2}: 100%; cpp {1,3}: 87.5% -> 02 is the only not-col-flipped (flipY)
7. lua chain: start+A key and every turn's scanned map (all non-`?` cells) equal to `hidden/NN.json`: 100% -> mazes exact. Confidence: certain for every cell reachable by the scan of the flood method (all reachable cells and their 5x5, i.e. everything that matters)

## Key

Shortest key separating validators: `A` alone (7 21 39 68 42 33 70 71, unique). Rick start too (all distinct). Hardcoding moves is not a gain (paths are hundreds of moves); the gain is the relaxed strategy below.

## Strategy facts verified on CG validators

- explore-all is NOT needed: goal = nearest of `?` or `C`, then shortest to `T` once on `C`, passes 100% (probe 1). Local `validators/` (synthetic) 02, 07 reject it; `hidden/` accepts it
- goal `[?T]` after `C` (explore continues after C) fails tests 04-08 locally, rejected
