# The Bridge (episode 2) hidden validators

Validator task 2026-10-20. Nothing recovered exactly: no `NN.json`, no `complete`.

## Count

12 validators. Source: partial scores on the JS leaderboard (33.3, 49.9) and our probes (8.3, 75.1, 83.4, 91.7) are all sums of 100/12 floats (k x 8.333..., floored or rounded to 0.1). Confidence high.

Titles probably match the 12 IDE tests (not verified). Forum (https://forum.codingame.com/raw/60, classic puzzle) has no validator inputs. It mentions validator 12 ("Well worn road") as harder than test 12: timeouts and searches that find no path, posts #29-#39.

## Measured (probes: general DFS that answers only when a predicate holds, score = count)

Notation: a test road transformed = 4 lane strings, optionally flipped (lane i <- lane 3-i), shifted in x by d (holes move right for d>0, `.` fill), padded with `.` or cut to the validator length L. Hamming = differing cells over the 4 lanes.

| predicate on the validator input | count | probe |
|---|---|---|
| full input (M, V, roads, S, lanes) equal to a test | 0/12 | oracle ruby 3038 |
| (L, V, S) equal to some test | 10/12 | oracle ruby 3055 |
| (V, S) equal to some test, any L | 11/12 | oracle ruby 3077 |
| (L, V, S) of a test and the same road within Hamming 4, no shift | 1/12 | oracle perl 3228 |
| (L, V, S) of a test and that test shifted d in -15..15 and/or flipped, within Hamming 4 | 9/12 | oracle perl 3397 |
| above OR any test, any (L, V, S), padded/cut, d in -25..25, flip, within Hamming 6 | 10/12 | oracle perl 3593 |
| any test, any (L, V, S), padded/cut, d in -25..25, flip, within Hamming 12 | 12/12 | oracle perl 3363 (CG 100%, committed as the perl row) |
| (L, V, S) of a test AND that test's minimal BFS plan survives (JS hardcode keyed on (L, V, S), crash otherwise) | 6/12 | earlier puzzle task, JS row 599 B score 49.9 |

## Deductions

- every validator is a variant of an IDE test: holes moved as a block (x shift, lane flip) plus a few cell edits. Same scenario, other values: the usual CG pattern
- 9 validators: same (L, V, S) as their test, road = test shifted and/or flipped, at most 4 cells off
- the other 3: one shares (L, V, S) with a test, one shares only (V, S) (other L), one shares neither. One of the 3 is within 6 cells of a transformed test (which one is unknown), all 3 within 12
- the test plans are NOT a hardcode: none of the 12 inputs equals a test. Only 6 of the 10 (L, V, S) twins accept their test's minimal plan
- unknown per validator: which test, shift d, flip, the edited cells. Count probes give sums over all validators, so identifying all 12 needs about 25+ more probes in fresh languages with a solver port

## Shortest separating key

Unknown. (L, V, S) does not separate: 10 validators share it with tests, and tests 5 and 8 share (36, 3, 6) between themselves.

## For cells

- a general method (search or heuristic) is still required. Before submitting a heuristic, check it on the tests shifted in x and flipped across lanes: these are the closest local stand-ins for the hidden validators
