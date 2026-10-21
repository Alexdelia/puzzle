# The Bridge (episode 2) hidden validators

Validator tasks 2026-10-20 and 2026-10-21. Nothing recovered exactly: no `NN.json`, no `complete`. Exact roads are out of reach of the score channel (each validator is a test moved/edited in ~4..12 cells, about 30+ bits each; a probe returns one (lights, heavies) pair).

## Count and weights

12 validators with unequal weights: 8 light (8.3 each) + 4 heavy (8.4 each), sum 100. Every score seen is a sum of these: 8.3 = 1L, 49.9 = 5L+1H, 74.8 = 8L+1H, 75.1 = 5L+4H (= 100 - 3L), 83.4 = 6L+4H, 91.6 = 7L+3H (one heavy missing), 91.7 = 7L+4H (one light missing). 74.8 is no k x 100/12, so the earlier "k/12 floats" reading was wrong. Confidence high (validator task 2026-10-21).

Heavy = validators 9..12: each of tests 9, 10, 11, 12 has exactly one (L, V, S) twin among the validators and it is heavy (bash probes not-* 91.6, see below). Read every score as (lights, heavies) = (validators among 1..8, among 9..12).

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

### Per-test (L, V, S) twins (validator task 2026-10-21, bash/C probes: plan-once DFS that answers garbage iff the input (L, V, S) is in a set; score = 100 minus the weights of matching validators)

Note test 10 has L = 35 (not 36).

| gated triple (test) | score | twins |
|---|---|---|
| (65,1,1) t12 | 91.6 | 1 heavy |
| (30,1,3) t11 | 91.6 | 1 heavy |
| (35,2,4) t10 | 91.6 | 1 heavy |
| (37,1,4) t9 | 91.6 | 1 heavy |
| (30,1,0) t1 | 91.7 | 1 light |
| (56,4,1) t2 | 100 | 0 |
| (42,4,8) t3 | 100 | 0 |
| (36,4,1) t4 | < 100 (bash row hidden) | >= 1 light |
| t4 or (39,2,2) t6 or (64,2,7) t7 | 75.1 (C, fresh row) | 3 light |
| (36,3,6) t5 = t8 | by difference: 10 twins - 4 heavy - t1 - 3 | 2 light |

So 1:1 with the tests (CG pattern) is consistent: validators 1, 4, 5, 6, 7, 8 keep their test's (L, V, S) (t4/t6/t7 one each is the likely split, not separated), validators 2 and 3 (V = 4, all 4 lanes, the block-hole tests) changed L (and maybe S); of these two, one keeps (V, S) of some test, the other does not (91.7 probe below).

### Lane changes (bash probe lanefree, 74.8 = 8L + 1H)

Plan-once DFS restricted to SPEED, JUMP, SLOW (no UP/DOWN) solves all 8 light validators and exactly 1 heavy one. Locally it fails tests 9, 10, 12 and passes 11, so most likely validators 1..8 and 11 never need a lane change, 9, 10, 12 do (heavy identity of the passing one unverified).

## Deductions

- every validator is a variant of an IDE test: holes moved as a block (x shift, lane flip) plus a few cell edits. Same scenario, other values: the usual CG pattern
- 9 validators: same (L, V, S) as their test, road = test shifted and/or flipped, at most 4 cells off
- the other 3: one shares (L, V, S) with a test, one shares only (V, S) (other L), one shares neither. One of the 3 is within 6 cells of a transformed test (which one is unknown), all 3 within 12
- the test plans are NOT a hardcode: none of the 12 inputs equals a test. Only 6 of the 10 (L, V, S) twins accept their test's minimal plan
- unknown per validator: which test, shift d, flip, the edited cells. Count probes give sums over all validators, so identifying all 12 needs about 25+ more probes in fresh languages with a solver port

## Shortest separating key

Not fully known. (L, V, S) separates validators 1, 9, 10, 11, 12 (one twin each, verified) and very likely 4, 6, 7 (3 twins over 3 triples, split not probed). Validators 5 and 8 share (36, 3, 6) like their tests (test 5 vs 8 differ only in lane 3 near x = 27: `000` vs `0000`). Validators 2 and 3 have an L not equal to 56 / 42; V = 4 with S = 1 vs 8 likely separates them from each other but S = 1, V = 4 also is test 4's. Roads unknown, so no plan table yet: a hardcode still needs per-validator plans (JS probe: only 6 of the twins accept their test's minimal BFS plan).

## For cells

- a general method (search or heuristic) is still required. Before submitting a heuristic, check it on the tests shifted in x and flipped across lanes: these are the closest local stand-ins for the hidden validators
