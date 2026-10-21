# Clojure: `(repeatedly(read)read)`, `(nth s 0 0)`, stable double sort

Verified on temperature (local tests 6/6 + validators, CG 100% at 95, 2026-10-21).

- `(repeatedly(read)read)` (22 B): reads the count, then exactly that many numbers, lazily. Never reads past the data, so `N = 0` gives an empty seq with no EOF throw. Beats `(next(read-string(str\[(slurp *in*)\])))` (40 B) and `(for[_(range(read))](read))` (27 B). Pairs: `(repeatedly(*(read)2)read)`
- `(nth s 0 0)`: first element, or 0 on an empty seq (8 B wrapper vs 12 B for `(or(first s)0)`). Works on lazy seqs
- `sort`/`sort-by` are stable (TimSort), so `(sort-by abs(sort > X))` gives min |t| first, with positive before negative on a tie. `min-key`/`max-key` keep the LAST of a tie, so `(apply min-key abs(sort X))` also works, but it throws on an empty seq
- speed: 9999 `read` calls take well under 1 s; a local timeout seen under machine load 42/8 was the JVM start alone (3-4 s for hello world)
