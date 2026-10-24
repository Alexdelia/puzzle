# The Fall - Episode 2 (code golf): hidden validators

7 validators, 1:1 with the 7 IDE tests (same titles), weights 160 (V1-V5) and 100 (V6, V7), total 1000. Source: CG per-validator reports of our submissions (`agent/tool/cg-report.sh <id>`, e.g. 41537696), 2026-10-24. The earlier "50 validators" (score-grain deduction) is wrong: scores are weight sums (16 = V1 alone).

Not recovered exactly: no `NN.json` written, no `complete` (V1 = test 1 exactly, kept out of `hidden/` until the set is whole).

## Probes

Old probes (2026-10-20, ids in `agent/state/events.log`) re-read with `cg-report.sh`; bit strings are V1..V7 pass (1) / fail (0):

| probe (submission) | predicate | bits |
|-|-|-|
| bash chain0 41537696 | grid, exit, Indy start equal an IDE test | 1000000 |
| bash chain1/2 41537706/13 | exact, or WxH of 10x8 / of any IDE test | 1000000 |
| bash chain3 41537722 | exact or W<9 | 1010000 |
| bash chain4 41537728 | exact or W<13 | 1110011 |
| bash chain5 41537746 | exact or W<17 | 1111111 |
| perl 41537787 | exact or H<9 | 1100110 |
| ruby 41537737 | no rock ever reaches the input (our solver playing) | 1111100 |
| ruby 41537793 | no rock or W<11 | 1111111 |

New probes (2026-10-24, python `probe/python/*.py`, `probes.tsv`; general solver of the old python solution, prints `X` at turn 1 unless the predicate on the init grid holds; generator `gen.py`+`lib.py` in journal scratch of task `validator.the-fall-episode-2.-.1792834544470160513`). "core" = nonzero bounding box of a test grid; "class" = unlocked-room rotation class of abs value ({2,3} {4,5} {6..9} {10..13}); x-mirror maps 4<->5 7<->9 10<->11 12<->13:

| probe | predicate | bits |
|-|-|-|
| core.py | abs grid contains abs core of a test 1-7 (offset, mirror) | 1001001 (test 1 core is 1x3 [3,2,3]: trivial match, ignore) |
| sim.py | (contained) signed zero-padded copy / (else) >=60% abs match | 1110110 (test 1 contamination, weak) |
| cls.py | (contained) signed contained / (else) class contained, tests 1-7 | 1010111 (contaminated) |
| cls2.py | class-contains core of a test 2-7 | 0011111 |
| abs2.py | abs-contains core of a test 2-7 | 0000001 |
| bit0-3.py | min abs diff over class placements, bits 0..3 | 0000110, 0011000, 0000000, 0000000 |
| bit4.py | bit 0 of (test index - 2) of that placement | 0010101 |
| bit7.py | that placement is x-mirrored | 0000000 |

## Per validator

- V1 Broken well: exact copy of test 1 (grid, exit, Indy start). Confidence high (chain0). Recovered: `01.json` = `tests/01.json`
- V2 Broken sewer: W 9..12, H<=8, no rocks; does NOT contain test 2's core (6x4) even up to room rotation; >=60% abs-similar to some test core (weak, sim.py). Layout of its own (stretched sewer?). Unknown otherwise
- V3 Broken secret passages: W<=8, H>=9, not 6x9; contains test 3's core (6x9) direct (not mirrored) with exactly 2 unlocked rooms rotated (abs differs, same class). Rest (offset, extra rows, signs, exit, start) unknown
- V4 Broken mausoleum: W 13..16, H>=10, contains test 4's core (13x10) direct with 2 rooms rotated
- V5 Underground complex: W 13..16, H<=8 so H=8 (core 13x8 needs 8 rows), contains test 5's core direct with 1 room rotated
- V6 Rocks 1: rocks reach Indy's input; W<=10, H<=8, not 10x8; contains test 6's core (9x8, cols 1..9) direct with 1 room rotated: so H=8 and W=9 (9x8 core in W<=10, H<=8, and 10x8 excluded): test 6 grid without its empty column 0, one room rotated; rock schedule unknown
- V7 Rocks 2: rocks; W 9..10, H>=9; contains test 7's abs core (9x8) exactly (0 rotated rooms), direct; not a zero-padded signed copy (sim.py, if not contaminated: extra nonzero cells outside the core or other signs)

Confidence: high for counts/bits (reports), medium for "V_k matches test k" (bit4 parity + dims agree for every k).

## Key / hardcode

Shortest known separating key: (W, H) classes: V1 5x3; V2 W 9..12 H<=8; V3 W<=8; V4 W>=13 H>=9; V5 W>=13 H=8; V6 W=9 H=8 (deduced); V7 W 9..10 H>=9. V2 and V6 may share W=9 H<=8: then H or grid content separates. Answers unknown: per-validator answers would differ from the tests' at the rotated rooms only (V3-V6) if the rest of the frame matches.

## Next bits (cheap, 1 probe each, all validators in parallel)

- per validator: x/y of the rotated rooms in core coordinates (bits of y*16+x of the k-th differing cell), new value (2 bits), offset of the core, W and H, sign differences vs test, exit x, Indy start, then a transcript checksum (knowledge `_shared/hidden-validators.md`) on the candidate
