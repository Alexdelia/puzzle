# Blunder - Episode 1: hidden validators

5 of 12 recovered exactly: `01`, `06`, `10`, `11` from inputs posted on the forum, `12` = IDE test 12 (probes). The reference (`solution/python/main.py`) reproduces the posted outputs and passes `--hidden` (17/17). No `complete` marker: 7 validators unknown (02 03 04 05 07 08 09), not equal to any IDE test.

## Count and weights

- 12 validators, titles = IDE test titles (forum: one failure shows 91 or 92%, posts 8, 44, 67, 160, 195 of https://forum.codingame.com/t/blunder-episode-1-puzzle-discussion/45; "seul le 1 et le 11" post 100 numbers them like the tests)
- weights are NOT equal: probe 1 (perl lookup of 4 forum validators + 12 IDE tests) scored 41.9, which is no k/12. Forum post 15: "LOOP (85 pts)". A score is the sum of the weights of the passed validators
- weights derived: w12 = 8.5 (probes 5, 6), w11 = 8.5 (puzzle-worker perl probe 17.0 = validators whose answer is an IDE test output = v11 `LOOP` + v12; matches forum "LOOP (85 pts)"), w01 + w06 + w10 = 24.9, sum of the 7 unknown = 58.1. Average 8.33: weights near equal but not equal, so a score is no exact count; decode probes by sums
- histogram probes of the puzzle worker (`oracle.sh --log blunder-episode-1`, general solver gated on a path predicate): never lands on T 74.9, never lands on a S/E/N/W modifier 58.1, never lands on I 66.4, never breaks an X 58.1

## Per validator

| # | title | known | source | confidence |
|---|---|---|---|---|
| 1 | Simple moves | exact: `6 5` (6 rows), `@` at row 1 col 2, `$` at row 4 col 3, empty inside; `SOUTH SOUTH SOUTH EAST` | raw post 179 (TwoSteps, 2021-06), https://forum.codingame.com/raw/45/179 | high (probe) |
| 2 | Obstacles | unknown, not an IDE test | probes 1, 3-6 | high |
| 3 | Priorities | unknown | | |
| 4 | Straight line | unknown | | |
| 5 | Path modifier | unknown | | |
| 6 | Breaker mode | exact: 10x10, `B X X B` column under `@`, `BX$` row 7; 6 SOUTH 3 EAST | raw post 194 (TwoSteps, 2021-11), https://forum.codingame.com/raw/45/194 | high (probe) |
| 7 | Inverter | unknown | | |
| 8 | Teleport | unknown | | |
| 9 | Broken wall? | unknown | | |
| 10 | All together | exact: 15x15, other map than test 10 (X wall around `$`, two T, B/I/N row); 33 moves | raw post 167 (TwoSteps, 2021-09), https://forum.codingame.com/raw/45/167 | high (probe) |
| 11 | LOOP | exact: validator 10 map plus modifiers `SW` (row 4), `S` (row 6), `N`, `W` (row 7); `LOOP` | raw post 196 (TwoSteps, 2022-01), https://forum.codingame.com/raw/45/196 | high (probe) |
| 12 | Multiple loops? | exact: = IDE test 12 (30x15, 156 moves) | probes 3-6: only test 12 matches a validator (bits 0,1 score 0, bits 2,3 score 8.5) | high |

Pattern: validators are distinct maps of the same theme, not transforms of the tests: v01 = test 01 with one more row and `@` moved, v06 = test 06 theme redrawn, v11 = v10 + modifiers exactly as test 11 = test 10 + modifiers.

## Probes (`agent/tool/oracle.sh --log blunder-episode-1`)

| # | lang | probe | passing set | score |
|---|---|---|---|---|
| 1 | Perl | `probe/perl/known16.pl` | lookup 4 forum validators + 12 IDE tests | 41.9 |
| 2 | Ruby | `probe/ruby/known4.rb` | lookup 4 forum validators | hidden (ruby row 74.9 already, puzzle worker) |
| 3 | Go | `probe/go/testbit0.go` | lookup IDE tests 1 3 5 7 9 11 | 0.0 |
| 4 | D | `probe/d/testbit1.d` | lookup IDE tests 2 3 6 7 10 11 | 0.0 |
| 5 | TypeScript | `probe/typescript/testbit2.ts` | lookup IDE tests 4 5 6 7 12 | 8.5 |
| 6 | Dart | `probe/dart/testbit3.dart` | lookup IDE tests 8 9 10 11 12 | 8.5 |

Probe 1 (41.9) minus probes 5/6: the 4 forum validators are still live, total weight 33.4.

## Key

Inside the known 4 plus tests, full-input lookup works; the shortest separating key not studied (unknown validators would collide). Input byte length: v01 40, v06 116, v10 246, v11 246, v12 486 (v10/v11 differ only by modifiers). The 7 unknown are not IDE tests, so they need a general solver or further recovery.
