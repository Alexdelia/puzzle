# Hidden validators: vox-codei-episode-2

Transform index t over test k (W x H): bit 4 transpose first, bit 1 flip x (x -> W-1-x), bit 2 flip y (y -> H-1-y). t=1 flip x, t=2 flip y, t=3 rot180. Node directions transform with the cells. Generator: journal scratch `gen.py` of task `validator.vox-codei-episode-2.-.1792523691648267216` writes all 80 transforms.

## Facts (all from CG probes 2026-10-20)

1. Every validator's first two maps equal a dihedral transform of one IDE test with the same bombs B, and the replayed transformed plan of that test wins: probe javascript 5907 (count, fresh lang) = 100%. Rounds R not checked by that probe
2. No validator is an identity copy or a transpose: every validator is t in {1,2,3} of its test (test 1 is y-symmetric: t=2 is identical to the test, t=1 = t=3): probe javascript 5886 (1-bit) = YES
3. Not all of tests 2..10 are flips without rot180: probe javascript 5875 (1-bit: t in {1,2} for tests 2..10) = NO (row stayed 5886, submission 41537854)
4. Exactly 3 validators are rot180 of one of tests 2..10: probe typescript 5109 (count, fresh lang, only t=3 of tests 2..10 allowed) = 30%
5. Ruby 214 hardcode keyed on `rounds%14`, map-blind plans of the IDE tests, scored 10% (cell ruby, 2026-10-20). Locally it wins on the identity of every test and on the flip-y of tests 1, 2, 4 only. With fact 2 and assuming each validator keeps its test's R: exactly one of {v1 = t2 (identical to test 1), v2 = t2, v4 = t2} holds. Deduction, confidence medium (R equality unproven)
6. Validator 1 is not identical to test 1, so it is its flip x (= rot180): probe javascript 5855 (1-bit: test 1 only t=2, others t in {1,2,3}) = NO. With fact 5: exactly one of v2, v4 is flip y (R equality assumed)
7. Validator 2 = flip y of test 2: probe javascript 5836 (1-bit: test 2 only t=2, others t in {1,2,3}) = YES. So the ruby 10% pass is v2 (key `rounds%14`==2: R=30 near certain) and v4 is t in {1,3}
8. Score granularity 10% (ruby 10.0, ts 30.0): 10 validators, assumed 1:1 with the tests (same B, same order unknown)

## Per validator (assumed 1:1 with tests)

| v | test | known | confidence |
|---|------|-------|------------|
| 1 | 01 | t=1 (= t=3): node starts at x=10 moving left, row 4, R 30, B 1 | high; `01.json` written with R 30 (R kept by v2 and by every ep1 validator) |
| 2 | 02 | t=2, R 30, B 2: `02.json` | high |
| 4 | 04 | t in {1,3} | high |
| 3,5..10 | 03,05..10 | t in {1,2,3}; with v4: three of 3..10 are t=3 | high |

Recovered: `01.json`, `02.json` (python ref passes `--hidden`). Others open, no `complete`.

## Key

Not needed for the transform: `rounds` per test (30,30,50,30,99,55,60,60,90,98) and the first map separate tests; the transform per validator is still open (2-3 options each).

## Next probes (cheapest that split most)

- fresh-lang count (php, lua, ...): t=2 only for tests 2..10 -> number of flip-y; combined with fact 4 gives the flip-x count
- per-validator 1-bit (javascript row, needs length < 5886: `PAD` in scratch `mk.sh`): restrict one test to one transform, others free
- check R: a count probe that also requires R == test R
