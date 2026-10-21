# Clojure: reduce over closures, merge-with accumulate

Verified on the-resistance local referee (2026-10-21).

- nested-loop state update: let `for` build one closure per step and apply them in order: `(reduce #(%2 %)init(for[i(range L)j(range i(+ i 77))]#(f % i j)))` beats `(reduce(fn[d[i j]](f d i j))init(for[...][i j]))` by 8 B (no `fn[d[i j]]`, no `[i j]` tuple). Cost: one closure alloc per step, ~1.3x slower on 7.6M steps
- add into a map key that may be missing: `(merge-with + d{j x})` 2 B shorter than `(update d j(fnil + 0)x)`, 3 B shorter than `(assoc d j(+(d j 0)x))`; slower (builds a map per call)
- `(frequencies(for[x w](apply str ...)))`: count map in one call (same-key words counted)
- `(for[c x b(rest(Long/toString n 2))]({\0\.\1\-}b))` 1 B shorter than `(mapcat #(map{\0\.\1\-}(rest ...))x)`
- `long-array` + `aset` with a boxed value goes through reflection: slower than a persistent map and longer
- string keys from `(apply str ...)` hash fast; char-seq keys (`(seq(subs ...))` vs lazy seq, equal and same hasheq) save 5 B but run ~4x slower
