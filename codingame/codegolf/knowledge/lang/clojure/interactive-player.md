# Clojure on interactive puzzles: ns Player, no gen-class

CG verified 2026-10-21 (power-of-thor test games 910670629, 910670752, 910670910, 910671093, 910671413; submits 153 and 143 at 100%).

- interactive puzzles compile the file as `Player` ("Compiling Player to /tmp"): `(ns Solution...)` fails with a syntax error, 0%
- the program runs through a `Bridge` class that requires the ns and calls `Player/-main` with ONE argument: `(defn -main[]` gives ArityException, use `(defn -main[_]`
- `:gen-class` not needed: header `(ns Player)(defn -main[_]` (26 B vs 38 for `(ns Solution(:gen-class))(defn -main[]`)
- no ns at all: "namespace 'Player' not found after loading '/Player'"
- top-level `(read)` (no -main): compile blocks on stdin, "Compilation took too long and has been interrupted"
- `(dotimes[i 99]` is 2 B shorter than `(doseq[i(range)]` for an endless-enough print loop
- local referee: build.sh now maps interactive to Player + Bridge; needs a CG_CLOJURE with `clojure-run-interactive` (older dev shells lack it)
