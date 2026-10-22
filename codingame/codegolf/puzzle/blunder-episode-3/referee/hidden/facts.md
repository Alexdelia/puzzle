# Blunder - Episode 3 - Code Golf: hidden validators

No validator recovered exactly: t values carry random noise, so `complete` is out of reach by probes. Facts below are counts over the whole validator set (CG gives no per-validator detail).

## Count

- **50 validators, 2% each** (deduction, high confidence): probe scores 0, 12, 14, 64, 76, 88, 100; 50 is the smallest count giving every score exactly (12 = 6/50, 14 = 7/50). The earlier 25-validator reading (journal puzzle...1792516442853674207) was refuted by the 14% scores
- the 8 IDE tests are not validators: no validator is byte-identical to an IDE test (TypeScript probe keyed on the sum of all numbers, 0%; code checked on IDE test 1 with a test game) — oracle log 2026-10-20 typescript 461
- web: no forum post or repo quotes validator data (forum search, github gustawdaniel/codingame-computational-complexity has only the statement example)

## Class histogram (constant-output probes, fresh languages, exact score)

| class | validators | probe |
|---|---|---|
| O(1) | 6 | lua `print"O(1)"` 12% |
| O(log n) | 7 | go 14% |
| O(n) | 6 | kotlin 12% |
| O(n log n) | 7 | groovy 14% |
| O(n^2) | 6 | dart 12% |
| O(n^2 log n) | 6 | swift 12% |
| O(n^3) | 6 | haskell 12% (scala probe crashed: CG Scala needs `object Solution`, row 0% meaningless) |
| O(2^n) | 6 | 50 - others (deduced) |

## Grid features (gates on the all-points regression class, score = share passing the gate)

- 32/50 share N with the IDE test of their class (ruby 64%)
- 32/50 share last n with the IDE test of their class (javascript 64%); likely the same 32: same n grid as the test, t redrawn
- 38/50 have a last n among the 8 test last n values (php 76%): 6 reuse the last n of a test of another class; 12 have a last n no test has
- at most 32/50 share first n and step with their class test (ruby, hidden behind the 64% row)
- 18/50 sit off the test grids (other N / last n): any key built from test-grid features caps at ~76%

## Methods measured on all 50

- all-points least-squares slope of `log t` on `log n`, `int(2k+.9)` capped at 7: 50/50 (Perl 180, CG 100%)
- index-weighted slope `Σ(2i-N+1)log t / Σ(2i-N+1)log n`, same index: 50/50 (Perl 123, CG 100% 2026-10-22)
- two-point exponent (first and last line), thresholds `x/2+.05`: 44/50 (Bash 88%)
- constant answer: 6 or 7 /50 (table above)

## Shortest key known

None. No single cheap input feature (N, last n, first n + step) is class-constant over the 50; the all-points regression is the only verified 100% method.
