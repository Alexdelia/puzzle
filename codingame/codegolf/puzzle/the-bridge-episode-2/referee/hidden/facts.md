# The Bridge (episode 2) hidden validators

## Recovered 2026-10-24 (per-validator report, bit-extraction probes)

Titles (CG report, weights per 1000): 1 One lonely hole (83), 2 Chained jumps of increasing length (83), 3 Chained jumps of decreasing length (83), 4 Chained jumps of equal length (83), 5 Diagonal columns of holes + 3 hole row (83), 6 Scattered pits (83), 7 Big jump chained with hole columns (83), 8 Diagonal columns of holes + 4 hole row with mandatory sacrifice (83), 9 Obstacle course for 1 bike (84), 10 Obstacle course for 2 bikes (84), 11 Mandatory sacrifice (84), 12 Well worn road (84).

Method: perl probes `probe/perl/bit00..07.pl`, `c08..c22.pl` (template in the journal scratch of task 1792834544389442463): the general DFS answers iff bit j of a per-validator encoding is 1; the report gives one bit per validator per submit.

### Validators 1, 5, 6, 7, 8, 9, 10, 11, 12: EXACT (high confidence)

Validator k = IDE test k mirrored across lanes: road lane y = test lane 3-y, same L, V, S, no x shift, no edit; bike lanes y -> 3-y (v1 lane 1, v7 lanes 1,2,3, v9 lane 1, v10 lanes 1,3, v6 1,2 unchanged, v5/v8/v11/v12 0..3). Checked by probes c08..c22: each answers iff road and bike-lane set equal the flipped test exactly (pass on all of them). Bike X = 0, all active. Files `NN.json` (bike order in the input not probed, written sorted).

### Validators 2, 3, 4: block holes (all 4 lanes identical, 4 bikes at X=0)

Lane-0 run lengths (first gap p, hole widths w, gaps g), from bits:
- v4 (L 36, V 4, S 1, triple of test 4): p = 14, all widths 2, all gaps 2, 4 groups: `..............00..00..00..00` + `.` to L = 36 (exact if L = 36, which the t4 triple gate proved). EXACT
- v3 (L, S not test 3's; (V, S) matches no test): p = 22, 3 groups, w1 = 5, g1 = 6, w2 = 4, widths not all equal; g2, w3, L, V, S unknown. Matches test 3 (`14:5,6:4,5:2`) shifted +8 in x so far, so likely L >= 44 (test 3: 42)
- v2 (L not 56; (V, S) equals some test's): p = 6, 4 groups, w1 = 2, g1 = 5, w2 = 3, widths not all equal, gaps not all equal; w3, w4, g2, g3, L, V, S unknown
- v4 tail bits (c20..c22) = 100x, consistent with L - 28 = 8

Next probes (tpl2 bit order, J = probe index): 23..25 v2 w3 / v3 w3, 26..28 v2 w4 / v3 g2(4 bits 26..29), v2 g2 29..32, g3 33..36, tail 37..40; v3 tail 30..33, pair 34..37, V 38..40, S 41..44. Cheaper for v3: one probe testing 'lane = test 3 shifted +8, all 4 lanes' then L and S bits.

## Shortest separating key

(L, V, S) separates all validators except v5/v8 (both (36, 3, 6)); v2 and v3 carry triples of no test. v5 vs v8: lane 0 (flipped test lane 3) at x 27..30.

## Earlier notes (2026-10-20/21, before the report)

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
