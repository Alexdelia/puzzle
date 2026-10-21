# Clojure: lazy stdin read, iterate as O(C) loop

Verified on roller-coaster (local 16/16, CG 100%, 2026-10-21).

- `[l c n & q](repeatedly read)` then `(take n q)`: reads whitespace-separated numbers lazily, 11 B shorter than `(read-string(str\[(slurp *in*)\]))` + destructuring when the count is known; never realize past EOF (`read` throws): bound with `take` before `cycle`/`drop`
- `(nth(iterate f x)c)` is a fine 1e7-step loop (no head retention, ~1 s); much shorter than `loop`/`recur` with a counter
- per-step cost matters at 1e7: `mapv` on a 2-vector or `juxt` inside the step is 3x slower than `update`; `memoize` lookup ~150 ns
- `filter` instead of `take-while` over a monotone finite seq (-4 B)
- `#(... (partial >= x) ...)` lets the outer fn be `#()` when the inner predicate would need a nested `#()`
