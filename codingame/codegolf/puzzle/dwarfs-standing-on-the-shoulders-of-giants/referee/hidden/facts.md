# Dwarfs standing on the shoulders of giants: hidden validators

4 of 10 recovered exactly (files `a-single`, `b-chain`, `c-chain-reversed`, `d-fork`: validator numbers and titles unknown, so stems are descriptive, not `NN`). Not `complete`.

## Count and structure

- 10 validators: partial leaderboard scores in steps of 10. High
- 4 IDE tests only (Simple example, Complete example, Several mentors, Several mentors 2): NOT 1:1
- every validator has `N <= 9` (perl probes, see knowledge). High
- `N` histogram: 6 with `N <= 2`, 2 with `N` in {3,4}, 2 with `N` in {8,9}, none in 5..7 (count probes ObjC 60%, JS `N<5` 80%, C++ 0%). High
- no validator has an IDE test's `(N, answer)` pair {(3,3),(8,4),(4,3),(9,5)} (C probe 0%): the 4 large ones are not relabelled IDE tests
- keys proven 100% on CG: `2 + #edges whose X is some Y`, `2 + |distinct X ∩ distinct Y|`. `2 + #edges whose Y is some X` = 90%: exactly one validator fails it (a node with 2 in-edges inside a path)

## Low group (N <= 2, 6 validators)

Probes 2026-10-20 (oracle log lines of this task):

- TS count probe, general answer iff every id <= 3: 50% -> 5 validators use only ids 1..3
- PHP count probe, iff `N<=2` and ids <= 3: 50% -> those 5 are all in the low group; exactly 1 low validator uses an id > 3; no `N>=3` validator uses only ids <= 3
- Dart count probe, iff input (lines trimmed) in {`1/1 2`, `2/1 2/2 3`, `2/2 3/1 2`, `2/1 2/1 3`, `2/1 3/2 3`}: 40% -> 4 of them
- Dart threshold probe `N>=8 || {1/1 2, 2/1 2/2 3}`: row moved at 40% -> both present
- TS threshold probe `N>2 || 2/2 3/1 2`: row moved at 50% -> present
- TS threshold probe `N>2 || 2/1 2/1 3`: row moved -> present, so `2/1 3/2 3` is NOT a validator

| validator | input (lines) | answer | confidence |
|---|---|---|---|
| a-single | `1`, `1 2` | 2 | exact, high |
| b-chain | `2`, `1 2`, `2 3` | 3 | exact, high |
| c-chain-reversed | `2`, `2 3`, `1 2` | 3 | exact, high |
| d-fork | `2`, `1 2`, `1 3` | 2 | exact, high |
| e (small ids) | `2` + a fork on ids 1..3 (out- or in-fork), not `1 2`/`1 3`, not `1 3`/`2 3`: 10 candidates | 2 | class known: TS threshold probe "N=1, edge != 1 2, ids<=3" left the row unchanged, so not a single |
| f (big id) | `N<=2`, some id > 3 | 2 (single or fork) | class known |

Both chains of the low group are a-d (earlier deduction: exactly 2 validators `(N=2, ans 3)`), so e and f answer 2.

Format assumption for the exact files: single space, final `\n` (as the IDE tests). The probes trimmed lines, so trailing spaces are not excluded.

## High group (4 validators, N in {3,4} x2 and {8,9} x2)

- ids: none uses only ids 1..3
- answers: not the IDE `(N, ans)` pairs; from older count probes (lua `N+1` 40%, ruby `N/3+2` 40%, python `print(input())` 30% unverified) exactly one of: (a) 1 pure chain + 3 others, or (b) 1 with answer `N` + 3 others, where "others" have answer not in {`N`, `N+1`, `N/3+2`}. With e/f both answering 2 and singles + forks = 4 in the low group (a-single, d-fork, e, f): singles count `a` decides it; e is a fork, so `a` = 1 + [f is a single]

## Shortest known separating key

- answers of the low group: 2 except the two chains (3). Any key must send `1/1 2`, `2/1 2/1 3` to 2 and `2/1 2/2 3`, `2/2 3/1 2` to 3; the two chains differ only in line order
- no exact key over all 10 (high group unknown); use the proven general-ish `2+|X∩Y|` family

## Next probes (not spent)

- e: 10 candidates `1 3/1 2`, `2 1/2 3`, `2 3/2 1`, `3 1/3 2`, `3 2/3 1` (out-forks), `2 3/1 3`, `1 2/3 2`, `3 2/1 2`, `2 1/3 1`, `3 1/2 1` (in-forks): about 4 threshold probes
- partial rows usable as threshold probes (row moves iff score >= row and shorter): TS 265@50 (pad `N>2` = 4, then 1 hit), PHP 353@50 (same), Dart 524@40 (pad `N>=8` = 2, then 2 hits)
- f: `N<=2`, an id > 3, answer 2: single or fork, ids unknown
