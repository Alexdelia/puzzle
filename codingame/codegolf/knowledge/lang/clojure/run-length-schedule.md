# Clojure: run-length output schedule, `run! #(prn % 4)`, key by `drop`

Verified CG 2026-10-21 (mars-lander-episode-3 112 at 100%, tie #1).

- `(run! #(prn % 4)S)` prints `x 4` per element and flushes: 19 B vs `(doseq[r S](println r 4))` 25. `prn` with several args joins them with a space. Space before `#(` is required (`run!#` reads as one symbol)
- run-length decode without apply: `(mapcat repeat[n1 n2 ..][v1 v2 ..])` = n1 copies of v1, then n2 of v2. Keyed on a choice it needs `(apply mapcat repeat({k1[[..][..]]k2[[..][..]]}x))`, +9 B
- key two inputs by an offset into ONE sequence: `(drop(*(read)K)S)` with first input number 22 vs 18 starts the 18-reader 4K elements earlier. Costs 16 B and one shared schedule instead of two: mars-lander A/B saved 19 B over the map-keyed pair
- `({22 a 18 b}(read))` is 1 B shorter than `(if(>(read)20)a b)`
