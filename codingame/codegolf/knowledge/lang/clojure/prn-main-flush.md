# Clojure: prn and -main arity

- `print` at the end of `-main` outputs nothing (stdout not flushed at exit); `prn` flushes and is 3 B shorter than `print` (`println` 2 longer). Verified locally and CG
- `(defn -main[]` works (zero args), `[&_]` reads as one symbol and fails with arity 0; `[& _]` also works but longer
- prefix set: `(set(mapcat #(reductions str %)lines))` (first element of each is a Char, still distinct, count right) (telephone-numbers 113)
