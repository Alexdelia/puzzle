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

## Shortest known

Method (verified local 20/20, tests + validators): two-point growth exponent, first and last measure only.

**CG refuted 2026-10-20: this method scores 88% on CG** (Bash cell, perl one-liner, `int(2k+.9)` = thresholds `x/2+.05`, local 20/20): one hidden validator (likely 1 of 8) misclassified. CG gives no per-validator detail. First/last points alone are not robust enough for CG: cells using it must change method (all points, more points, other thresholds), and local validators do not reproduce the failure, so a validator closer to CG's is needed before trusting any variant.

- `k = log(t_last/t_first) / log(n_last/n_first)`; middle lines ignored, `N` only skipped
- measured k (8 tests + 12 validators, min..max): `O(1)` -.007..-.001, `O(log n)` .132, `O(n)` .869..0.874, `O(n log n)` 1.20..1.21, `O(n^2)` 1.80..1.81, `O(n^2 log n)` 2.25..2.26, `O(n^3)` 2.90..2.91, `O(2^n)` 5.06..6.17
- uniform thresholds `x/2 + e`, x = 0..6, any `e` in (.03, .13) separates all 8; `e` .05..0.1 balanced (margin ~.05 in k). class index = count of thresholds passed = `int(2k + 1 - 2e)` capped at 7
- no log needed: index = count of x in 0..6 with `t_last/t_first > (n_last/n_first)^(x/2+.05)`; cap at 7 comes free (2^n passes all 7)
- with log: `int(2*k+.9)`, 2^n gives 10..13 so needs cap (`min(7,..)`, Python `[:int(2*k+1.9)][-1]`, JS `arr[i]||"2^n"`)
- output table in class order, index 0..7: `1,log n,n,n log n,n^2,n^2 log n,n^3,2^n`, wrap in `O(` `)`
- needs: float division and pow (or log); integer-only langs (bash) need awk/bc or a fixed-point trick

Lengths reached: Python 156 (count form, no import), 160 (log form). Top Python 97: ~60 below this method.

Unverified suspicion: 97-ish tops hash test features (N, first line) into the table; input reading + table + print already cost ~83 in Python, leaving ~14 for the class pick. N alone fails (3 tests have N=100); first line `n t` differs per test (`10 200000`, `5 552970`, `5 341`, `34 988`, `34 89`, `34 31261`, `11 51`, `2 19`). Such a hash only passes if hidden validators share those features; our generated validators would reject it, and `submit.sh` requires them. Not tried.

Rejected:

- last point only (`t/n`, `log t/log n`): classes overlap (`O(1)` vs `O(n)`)
- last two points: exponent noisy (log test gives 14.5)
- middle and last: `O(1)` .08..0.14 vs `O(log n)` .10..0.11 overlap
- linear contiguous mapping with step != 2 or the inverse `c/(k+d)`: wider index range, no gain (search: journal scratch `srch.py`)
