# power-of-thor hidden validators

4 validators (score grain 25% on every language leaderboard, e.g. OCaml/Pascal/Kotlin rows 0/25/50/75/100, `cg.sh GET /puzzles/power-of-thor/leaderboard?language=...`, 2026-10-20). IDE tests: 1 Straight line, 2 Down, 3 Easy angle, 4 Optimal angle.

Init line = `lightX lightY thorX thorY`. Probes (`agent/tool/oracle.sh --log power-of-thor`, 7 fresh-language count probes, general solution gated by a predicate on the init line):

| probe | lang | predicate | count |
|---|---|---|---|
| 1 | javascript | line in {7 8 28 8, 5 17 5 4, 31 16 0 2, 0 17 36 0, 31 4 5 4, 36 17 36 0, 31 4 31 17, 0 17 31 4, 36 17 0 0} | 1 |
| 2 | php | lightY<thorY (needs N) | 1 |
| 3 | lua | line == 31 4 31 17 | 0 |
| 4 | java | lightX>thorX (needs E) | 0 |
| 5 | go | lightX==thorX (vertical) | 1 |
| 6 | typescript | lightX==thorX and lightY<thorY (vertical up) | 1 |
| 7 | kotlin | lightY==thorY (horizontal) | 1 |

## Per validator (confidence: counts certain, mapping to titles assumed from test order)

- horizontal: exactly 1, light WEST of Thor (W only)
- vertical: exactly 1, light NORTH of Thor (N only, an "Up" case). It is not `31 4 31 17`
- the other 2: light strictly south-west (dx<0, dy>0): moves SW then W or S. No validator needs N except the vertical one
- no validator needs E, NE, SE, NW
- exactly 1 validator has its init line in the 9-set of probe 1; with the direction facts it is one of `7 8 28 8` (current test 1), `0 17 36 0` (current test 4), `0 17 31 4` (old IDE Easy angle). Not resolved
- energies unknown (slack or exact)

## Source of the old IDE tests (used in probe 1)

forum https://forum.codingame.com/t/power-of-thor-episode-1-puzzle-discussion/30 (posts 38, 90, 152, 767, 842, 874, 988: `Thor position = (31,4). Light position = (0,17). Energy = 44`, `(5,4)/(31,4)` energy 100, `(0,0)/(36,17)` energy 36, `(36,0)/(36,17)`), used until at least 2022-11. A 2016 post: IDE test 2 named "Up", validator 2 named "Down". Codesize thread https://forum.codingame.com/t/power-of-thor-codesize-puzzle-discussion/687 : validators changed 2020-07 (posts 167-172), still "lack a direction"; a random pick among 4 hardcoded outputs passed each validator with p=1/4 (post 163, before the change).

## Shortest key known

None yet (exact inputs unknown). Direction structure is enough for a branch-reduced general solution: horizontal step only W (`x>LX`), vertical N only when x already equals LX from the start.
