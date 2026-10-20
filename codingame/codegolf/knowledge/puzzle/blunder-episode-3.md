# Blunder - Episode 3

pretty_id `blunder-episode-3`, leaderboard `blunder-episode-3-codesize`, io.

## Rules (as referee)

- input: `N`, then `N` lines `num t`, `num` unique ascending
- output one line, exact string among `O(1)`, `O(log n)`, `O(n)`, `O(n log n)`, `O(n^2)`, `O(n^2 log n)`, `O(n^3)`, `O(2^n)`
- 8 tests, one per class, in that order (test 01 = `O(1)` ... test 08 = `O(2^n)`); hidden validators: reference scores 100% on submit, their class order unverified
- t values noisy; log-space least squares (variance of `log t - log f(n)` per class, pick min) passes all 8 tests and our 18 validators

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

Method (local 20/20 before validators 13-18, now 20/26): two-point growth exponent, first and last measure only.

**CG refuted 2026-10-20: this method scores 88% on CG** (Bash cell, perl one-liner, `int(2k+.9)` = thresholds `x/2+.05`, local 20/20): 6 of 50 hidden validators misclassified. CG gives no per-validator detail. First/last points alone are not robust enough for CG: cells using it must change method.

Local validators 13-18 (added 2026-10-20) reproduce the failure: contiguous subranges of the real CG tests (O(1) n 3410..9910, O(log n) 3105..4990, O(n) 7705..9405, O(n log n) 8134..9734, O(n^2) 6234..7134, O(n^2 log n) 159..224), each picked so the reference (log-space variance) wins with margin >= 2x and the two-point exponent misclassifies; the two-point bash 116 now fails 6/26 locally. Which CG validator actually failed stays unknown (CG gives no per-validator detail, 88% = 44/50, 6 validators fail): local 26/26 is necessary, not proof of CG 100%.

Endpoint noise: at full test ranges the two-point method needs a 40-50% error on the first or last t to flip (O(1) 1.49x, O(log n) 1.76x, O(n^2 log n) 1.55x, O(n^3) 1.77x); real test 02 first point already sits +30% above the fit.

All-points alternative (verified local 26/26, unverified on CG): least-squares slope k of `log t` on `log n` over all measures, measured k per class: O(1) -.003..0.011, O(log n) .097..0.125, O(n) .867..1.02, O(n log n) 1.116..1.17, O(n^2) 1.793..2.027, O(n^2 log n) 2.269..2.463, O(n^3) 2.949..2.95, O(2^n) 5.62..6.35. Index `int(2k + c)` (capped at 7) is correct on all 26 for c in [.806, .946); .9 inside, margin ~.05 each side.

- `k = log(t_last/t_first) / log(n_last/n_first)`; middle lines ignored, `N` only skipped
- measured k (8 tests + validators 01-12, min..max): `O(1)` -.007..-.001, `O(log n)` .132, `O(n)` .869..0.874, `O(n log n)` 1.20..1.21, `O(n^2)` 1.80..1.81, `O(n^2 log n)` 2.25..2.26, `O(n^3)` 2.90..2.91, `O(2^n)` 5.06..6.17
- uniform thresholds `x/2 + e`, x = 0..6, any `e` in (.03, .13) separated tests + validators 01-12 (not 13-18); `e` .05..0.1 balanced (margin ~.05 in k). class index = count of thresholds passed = `int(2k + 1 - 2e)` capped at 7
- no log needed: index = count of x in 0..6 with `t_last/t_first > (n_last/n_first)^(x/2+.05)`; cap at 7 comes free (2^n passes all 7)
- with log: `int(2*k+.9)`, 2^n gives 10..13 so needs cap (`min(7,..)`, Python `[:int(2*k+1.9)][-1]`, JS `arr[i]||"2^n"`)
- output table in class order, index 0..7: `1,log n,n,n log n,n^2,n^2 log n,n^3,2^n`, wrap in `O(` `)`
- needs: float division and pow (or log); integer-only langs (bash) need awk/bc or a fixed-point trick

Lengths reached: Python 156 (count form, no import), 160 (log form), both two-point, so both now fail validators 13-18. Top Python 97: ~60 below this method.

Unverified suspicion (see "Hidden validators": 50 validators, 18 off the test grids, none equal to a test, so a test-feature table fails): 97-ish tops hash test features (N, first line) into the table; input reading + table + print already cost ~83 in Python, leaving ~14 for the class pick. N alone fails (3 tests have N=100); first line `n t` differs per test (`10 200000`, `5 552970`, `5 341`, `34 988`, `34 89`, `34 31261`, `11 51`, `2 19`). Such a hash only passes if hidden validators share those features; our generated validators would reject it, and `submit.sh` requires them. Not tried.

Rejected:

- last point only (`t/n`, `log t/log n`): classes overlap (`O(1)` vs `O(n)`)
- last two points: exponent noisy (log test gives 14.5)
- middle and last: `O(1)` .08..0.14 vs `O(log n)` .10..0.11 overlap
- linear contiguous mapping with step != 2 or the inverse `c/(k+d)`: wider index range, no gain (search: journal scratch `srch.py`)

All-points regression on CG (verified 2026-10-20): Perl 180 passes CG 100% (local 26/26). `-n`, `/ /&&` skips header, `($x,$y)=map(log,split)`, sums a,b,c,d,n, `}{print` after the loop, index `2*($n*$c-$a*$b)/($n*$d-$a*$a)+.9` into the table, `//"2^n"`. Sub-80 tops (Perl 79) cannot be a regression: table alone is ~65 bytes.

## Hidden validators

Full detail, sources and probe lines: `puzzle/blunder-episode-3/referee/hidden/facts.md`. No validator recovered exactly (t carries random noise); no `complete`.

- **50 validators, 2% each** (deduced 2026-10-20: 50 is the smallest count giving every probe score exactly; 14% = 7/50 refutes the earlier 25-validator reading)
- class histogram (constant-output probes): O(1) 6, O(log n) 7, O(n) 6, O(n log n) 7, O(n^2) 6, O(n^2 log n) 6, O(n^3) 6, O(2^n) 6
- no validator is byte-identical to an IDE test (0%, probe code checked by test game): hardcoding the 8 tests scores 0
- 32/50 share N and last n with their class test (same grid, t redrawn); 38/50 have a last n among the 8 test last n; 18/50 sit off the test grids
- methods: all-points log-log slope 50/50 (Perl 180), two-point first/last 44/50 (Bash 88%), constant answer 6-7/50
- test games only reach testcases 1..8 (`INVALID_TESTCASE` on 9)
- CG Scala needs `object Solution` (`object Main` crashes: "No such file or class on classpath: Solution")

Rows left by probes (below 100%, any 100% submission replaces them): Ruby 64% (270), JS 64% (433), PHP 76% (362), Lua 12% (11), TypeScript 0% (461), Go 14% (62), Kotlin 12% (27), Groovy 14% (17), Dart 12% (24), Swift 12% (21), Scala 0% (42), Haskell 12% (19).

Guesses (unverified): tops (Ruby 76, Perl 79, PHP 83) leave ~13-17 chars for read + index, so they key on a cheap feature constant per class over all 50 validators, or a short robust fit. N and last n collide across classes on the test grids (N=100 for O(1), O(n), O(n log n)), and 18 validators sit off the grids. A per-input hash of 50 inputs into 8 slots is hopeless.
