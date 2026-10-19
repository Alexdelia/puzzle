# Blunder - Episode 3

pretty_id `blunder-episode-3`, leaderboard `blunder-episode-3-codesize`, io.

## Rules (as referee)

- input: `N`, then `N` lines `num t`, `num` unique ascending
- output one line, exact string among `O(1)`, `O(log n)`, `O(n)`, `O(n log n)`, `O(n^2)`, `O(n^2 log n)`, `O(n^3)`, `O(2^n)`
- 8 tests, one per class, in that order (test 01 = `O(1)` ... test 08 = `O(2^n)`); hidden validators: reference scores 100% on submit, their class order unverified
- t values noisy; log-space least squares (variance of `log t - log f(n)` per class, pick min) passes all 8 tests and our 12 validators

## Traps vs statement

- statement says `5 < num`, test 08 (`O(2^n)`) starts at `num=2`, test 07 at `11`: `log(log n)` must stay defined at n=2 (fine), n=1 would break it
- test 02 (`O(log n)`) has `N=999`, ~12.7 kB of input; others N 24..100
- `O(1)` test: t around 200000 with noise, not constant

## Input quirks

- final newline present, no trailing spaces, no CR

## Tooling notes

- CG test output capped ~4 kB: `capture-io.sh` truncated test 02; captured in chunks (zlib+b85 of stdin, 4000-char slices, one game per chunk)

## Reference

- Python 748 (plain, CG 100% 2026-10-19, #124 tie #123); top Python 97 (Wazemo)
