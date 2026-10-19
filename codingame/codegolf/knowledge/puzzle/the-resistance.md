# The Resistance

pretty_id `the-resistance-codesize`, io.

## Rules (as referee)

- input: Morse line (`.` and `-` only), `N`, then N uppercase words, one per line
- output: number of ways to split the whole Morse line into dictionary words, as integer; exact match of line 1
- distinct words with the same Morse encoding count separately (test 05: CAT/KIM/TEXT/TREM/CEM, answer 125): a set of encodings undercounts
- R < 2^63 by statement: needs 64-bit ints, not 32 (test 04 = 57330892800, test 06 = 2971215073)
- words up to 19 letters, so a match spans at most 76 Morse symbols

## Tests

- 01-03 tiny, 04: L 9884, N 9444, words mostly 5-6 letters, answer 57330892800
- 05 same-encoding words, 06: 52 dots with dictionary `E I` (Fibonacci, 2971215073)
- io limit 6000 ms per test (shared probe `knowledge/lang/_shared/io-time-limit.md`, not re-probed here)

## Traps

- tests 05 and 06 have NO final newline on the last word (captured byte-exact through lzma+b85); 01-04 end with `\n`. A per-line reader that needs `\n` loses the last word; `split()` is safe
- `capture-io.sh` truncates test 04 (CG output cap ~4 kB): captured as lzma+b85 slices, 12 games (journal `referee.the-resistance.*`, scratch `capture.sh`)
- naive O(L * 76) substring dict lookups are fine for test 04 in Python (reference with a trie: 0.08 s on 04, 0.9 s on validator L 99029 N 40000)
- validators (ours) cover: R exactly near 2^63 (Fibonacci 91 dots, dashes, alphabet), R = 0 with trailing unmatched symbol, single letter without final newline, L ~ 99000 with N 40000

## Shortest known

- ours Python 991 (plain reference, 2026-10-19), rank #259 tie
