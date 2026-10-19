# Dwarfs standing on the shoulders of giants

pretty_id `dwarfs-standing-on-the-shoulders-of-giants-codesize`, io.

## Rules (as referee)

- input: line `N`, then `N` lines `X Y` (X influences Y), any order
- graph is a DAG (no cycle, no self edge), may be disconnected, several roots
- output: number of nodes (not edges) on the longest path, single line
- constraints: `0 < N < 10000`, `0 < X, Y < 10000`

## Tests (captured 2026-10-19)

- 4 tests, 3, 8, 4 and 9 edges, ids 1..11, answers 3 4 3 5
- tests 01-03 are the statement examples; 04 "Several mentors 2" is an extra small case
- single space between X and Y, one final `\n`, no trailing space, no blank lines
- local validators 01-06: single edge with X > Y (answer 2), ids up to 9999, longest vs shortest path trap, several components, layered DAG with skip edges (39 edges), random DAG (19 edges, answer 7)

## Timing

- io limit 6000 ms per test, from `knowledge/lang/_shared/io-time-limit.md`, not re-probed here
- sizes in tests are tiny: plain recursive DFS without memo is fast on every local case

## Unverified

- hidden validator sizes unknown; local validators stay under 40 edges and depth 7, so a deep-recursion solution passes locally, risk only if CG hides a long chain
