# Genome Sequencing

pretty_id `genome-sequencing-codesize`, leaderboard `genome-sequencing-codesize`, io.

## Rules (as referee)

- input: line `N` (`0 < N < 6`), then `N` lines of `ACGT`, each 1-10 chars; output: one integer, length of shortest common superstring
- duplicates and strings contained in others are legal (tests 04/05 have `GAT` inside `AGATTA`); a pure suffix/prefix merge without containment check fails them
- max-overlap merge along every permutation is exact once contained strings are dropped (or skipped when already inside the merged string); fixed input order is not enough (tests 03/04/05 fail)
- io limit 6000 ms per test (shared rule, see `lang/_shared/io-time-limit.md`); 5! = 120 orders, any brute force over orders is far below it

## Input quirks (captured 2026-10-19)

- every test ends with a final newline, no CR, no trailing spaces
- CG tests hold only N 2-3, max string length 7; validators may go to N 5, length 10 (local validators do)

## Local validators

- 01 N=1, 02 N=5 length 10 with overlap 1 only (48), 03/09 duplicates, 04/07/08 containment chains, 05 N=5 length 10, 06 three-way chain, 10 cyclic overlaps where the best order uses overlap 4 everywhere (8, a first-fit order gives 11)
