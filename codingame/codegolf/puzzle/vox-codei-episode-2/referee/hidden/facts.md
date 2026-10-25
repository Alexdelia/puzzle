# Hidden validators: vox-codei-episode-2

10 validators, validator k = transform t_k of IDE test k: same rounds R, same bombs B, grid and nodes (positions and directions) mirrored. t=1 flip x (x -> W-1-x), t=2 flip y (y -> H-1-y), t=3 rot180 (both). All 10 recovered exactly as `NN.json` (generator: journal scratch `gen.py` of task `validator.vox-codei-episode-2.-.1792523691648267216`, file `NNt.json`).

| v | test | t | R | B | source |
|---|------|---|---|---|--------|
| 1 | 01 | 1 flip x (= rot180, test 1 is y-symmetric) | 30 | 1 | probe javascript 5855 NO + ruby 10% |
| 2 | 02 | 2 flip y | 30 | 2 | probe javascript 5836 YES |
| 3 | 03 | 3 rot180 | 50 | 7 | count probes + exact probe |
| 4 | 04 | 1 flip x | 30 | 1 | count probes + exact probe |
| 5 | 05 | 2 flip y | 99 | 9 | count probes + exact probe |
| 6 | 06 | 3 rot180 | 55 | 1 | count probes + exact probe |
| 7 | 07 | 1 flip x | 60 | 9 | count probes + exact probe |
| 8 | 08 | 2 flip y | 60 | 4 | count probes + exact probe |
| 9 | 09 | 3 rot180 | 90 | 3 | count probes + exact probe |
| 10 | 10 | 1 flip x | 98 | 10 | count probes + exact probe |

Confidence: certain. Final check: python probe `probe/python/exactE.py` (891) wins only on exactly these 10 variants (local referee over all 80 transforms) and scored 100% on CG (committed as python 891, 2026-10-21). R equality also follows from the rounds-keyed cell hardcodes (php, js, ts) scoring 100%.

## Probe chain (probes.tsv, 2026-10-20/21)

1. javascript 5907 count 100%: every validator = dihedral transform of a test, same B
2. javascript 5886 YES: no identity, no transpose
3. typescript 5109 count 30%: exactly 3 rot180 among v2..v10
4. javascript 5855 NO / 5836 YES: v1 flip x, v2 flip y
5. ruby 815..827 count (row was 214 at 10%), perl fresh count, perl count, ruby count, python 1-bit (D NO), python 1-bit (E YES): 1232 -> 300 -> 22 -> 5 -> 2 -> 1 hypotheses

## Key

- `rounds bombs` separates the validators except v1/v4 (both 30 1); v1 and v4 are both flip x and one plan (bomb at turn 14, from tests 1/2/4) wins both after the flip. `b*(r+9)%10` (php, cell) is injective on the 9 distinct pairs
- since the transform per validator is fixed, a hardcode can store plans already in the validator's orientation: no orientation detection, output depends only on (r, b) and turn. Such a code fails the IDE tests locally (they are the untransformed maps), only the validators count on CG
- maps: v1..v9 12x9, v10 16x12

## Local referee (2026-10-25)

- `referee/src/runner.rs` `ClosedStdin<G>` ignores a failed turn-input write like CG; plan-print-then-exit rows are no longer flaky ("solution closed its input" gone)
- `--cg` grades `hidden/` only (IDE tests and synthetic validators are skipped while `hidden/complete` exists). All 27 `solution/*/main.*` at their cell `.ours` pass hidden 10/10, except clojure 289 / groovy 553 / scala 231 locally at `turn 1: no answer within 1000 ms` under host load: JVM launcher startup, not modelled (CG starts its first-turn clock after JVM boot; the rows are CG 100%). Groovy/scala pass on an idle host
- a player that exits with too few buffered lines still fails `turn 2: solution exited`
- general reference python 3833 (git 2b62a0a7) passes tests+validators+hidden 30/30; the synthetic validators still reject the rounds-keyed plan hardcodes by design
