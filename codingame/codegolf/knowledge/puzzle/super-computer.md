# Super Computer

pretty_id `super-computer-codesize`, leaderboard `super-computer-codesize`, io.

## Rules (as referee)

- input: `N`, then `N` lines `J D` (start day, duration). Output: max count of pairwise non-overlapping calculations
- calculation covers days `J .. J+D-1` inclusive: two are compatible iff `J1+D1 <= J2`. Strict `<` is wrong (test 2 gives 3 instead of 4; test 3 gives 7829 instead of 7879)
- classic interval scheduling: sort by end `J+D`, take greedily
- io limit 6000 ms per test (shared rule, `lang/_shared/io-time-limit.md`, not re-probed); plain Python greedy with sort takes 170 ms on N=99999 locally

## Tests (CG, probed 2026-10-19)

- 01, 02 = statement examples, answers 3 and 4
- 03 "Large number of scientists": N=99999, 1078215 chars, J in 1..999988, D in 1..999 (all 999 values present), unsorted, uniform random, 99992 distinct pairs (duplicate lines exist), 95225 distinct J, answer 7879
- 03 is too big to capture (CG output cap ~4 kB, 580 kB lzma). Local `tests/03.in` is synthetic: same N and distribution (Python `random.Random(0)`), answer 7978. CG 03 sha256 `8dce459ea33cb6e4f7e026a6efed9c070896fe6697ece3d15881a0add5f8f594`
- end day `J+D` reaches up to ~1000997: above 1e6, fits int32

## Input quirks

- final newline present (all 3 tests), no trailing spaces, no CR, single space separator
- N = number of following lines: reading all lines after the first works

## Validators (local)

- edge cases: N=1, identical intervals, touching chain (`J+D == next J`, all fit), overlap by one day, nested long interval, many ties, ends past 1e6, N=99999 all disjoint (answer = N), heavy overlap
