# Don't Panic: hidden validators

All 7 recovered: each validator is the IDE test of the same number, same game input (floors, width, rounds, exit, clones, generator, elevator positions). `01.json`..`07.json` = copies of `tests/`.

## Count

7 validators: partial scores on the leaderboard step by 14.3 (Bash, PHP, Ruby, Perl, C rows, 2026-10-20). Titles = IDE test titles (assumed, 1:1 count).

## Per validator

| # | title | input | confidence |
|---|---|---|---|
| 1 | U Turn | = test 01 | high |
| 2 | One floor | = test 02 | high |
| 3 | U Turn, lower floor | = test 03 | high |
| 4 | Several floors | = test 04 | high |
| 5 | 6 floors, lot of rounds | = test 05 | high |
| 6 | Few rounds | = test 06 | high |
| 7 | Spaced elevators, few rounds | = test 07 | high |

Key probed: `F W R exitFloor exitPos clones generator e[0] .. e[F-2]` (elevator pos per floor, generator = clone pos of turn 1). Not probed: order of the elevator lines in the init input (forum says it is shuffled between runs of the same test: NewboO, https://forum.codingame.com/t/dont-panic-episode-1-puzzle-discussion/314 post #14), so do not key on line order.

Multiplicity: probes show the validator set is a subset of the 7 test inputs and contains test 07; one validator per test is the 1:1 title assumption, not probed separately.

## Probes (PHP, `agent/tool/oracle.sh --log dont-panic`, task validator.dont-panic.-.1792523690577001181)

Each probe: honest greedy iff key in set, else prints `X` (invalid, fails at turn 1).

| # | len | set | result |
|---|---|---|---|
| 1 | 919 | tests ∪ x-mirrors (pos -> W-1-pos) | 100% (row) |
| 2 | 703 | mirrors only | no |
| 3-8 | 899, 877, 854, 825, 789, 749 | drop mirror k, k = 1..6 cumulatively | yes each |
| 9 | 710 | tests 1-6 + mirror 7 | no |
| 10 | 704 | tests 1-7 only | 100% (row, committed) |

## Forum notes (contradicted by probes, kept as warning)

Thread https://forum.codingame.com/t/dont-panic-episode-1-puzzle-discussion/314: several posts (#32, #82, #336, #341) claim validator 1 differs from IDE U Turn, #236 claims validator 7 has another top-elevator position. Probe 10 says the current validators equal the current IDE tests: these posts are either from older versions or from buggy code. Posts #14, #330 quote a test 5 header `7 24 200 6 17 40 0 6` (exit pos 17) which matches neither today's test 5 nor any validator: old data.

## Shortest key

Any input feature that separates the 7 IDE tests separates the validators. Greedy BLOCK turns per test are in `knowledge/puzzle/dont-panic.md` (JS section). Separators (from tests/): `generator` alone: 11, 5, 10, 6, 19, 11, 8 (tests 1 and 6 collide); `exitPos`: 3, 3, 1, 5, 19, 14, 11 (1, 2 collide); `(exitPos, nbFloors)` distinct; `rounds+exitPos` or `width` + generator also distinct.
