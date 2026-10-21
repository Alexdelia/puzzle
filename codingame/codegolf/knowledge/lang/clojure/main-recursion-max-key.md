# Clojure: `-main` self-call loop, `max-key` key-on-index

Verified CG 2026-10-21 (the-descent 75 at 100%, tie #1).

- endless turn loop: `(defn -main[_](-main(prn ..)))` wraps the turn body as the recursion argument: 7 B overhead vs `(while 1 ..)` 9 and `(dotimes[i 99] ..)` 15. `prn` returns nil, fed back as the ignored `_`. Stack depth = turn count, fine for ~100s of turns; EOF on `(read)` throws, crash after last output is harmless (`../_shared/crash-exit.md`)
- argmax over 8 lines without an array: `(apply max-key(fn[_](read))(range 8))`. 1.11 `max-key` calls the key exactly once per element, in order; ties go to the last max
- `#(read)` fails (arity 0, max-key passes 1 arg); `(partial read)` / `(comp read)` pass the index to `read` as a stream: ClassCastException. `(fn[_](read))` 13 B is the shortest 1-arg reader found
- local referee: needs a CG_CLOJURE with `clojure-run-interactive` (`agent/tool/lang/package.sh clojure`) and a scratch referee copy with FIRST_TURN raised (JVM start ~1.3 s > 1000 ms); `submit.sh` refuses on that local timeout, send through `oracle.sh`
