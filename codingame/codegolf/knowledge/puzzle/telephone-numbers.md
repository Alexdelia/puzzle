# Telephone Numbers

pretty_id `telephone-numbers-codesize`, leaderboard `telephone-numbers-codesize`, io.

## Rules (as referee)

- line 1 `N`, then `N` lines of digits; answer = number of distinct non-empty prefixes over all numbers (trie node count without root)
- duplicate numbers count once; a number that is a prefix of another adds nothing
- statement: `0 <= N <= 10000`, `2 <= L <= 20`; N=0 never seen in tests, unknown in validators (not in our validators)
- io limit 6000 ms per test (shared rule, `lang/_shared/io-time-limit.md`, not re-probed); plain Python set of prefixes takes 60 ms on test 5

## Input (captured 2026-10-19)

- final newline present in all 5 tests, no CR, no spaces, digits only
- tests 1-4 are the statement examples (answers 10, 20, 10, 28)
- test 5: N=10000, lengths 3..14 (5320 of length 10, none above 14 despite L <= 20), 84 duplicate lines, answer 45512
- `capture-io.sh` truncates test 5 (92778 chars, CG stderr cap ~4 kB): `referee/capture/capture.sh 5` captures lzma+b85 in 3800-char chunks, 14 games

## Validators (ours)

- mirrors of tests 1-4 with other values, a long list with test 5 length histogram, 10000 numbers of length 20, 50 copies of one 2-digit number, all 100 2-digit numbers plus 3-digit ones, shuffled prefix chain of lengths 2..20, many numbers with leading `0`/`00`
- numbers must stay strings: leading zeros matter (`0123` and `123` differ), int conversion is wrong
