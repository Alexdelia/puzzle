# Clojure on interactive puzzles: `ns Player`, `-main` takes 1 arg

CG verified 2026-10-21 (power-of-thor test games 910670828, 910670850, 910671093, 910671218, referee.power-of-thor journal 1792607297208713249).

- interactive puzzles compile the file as `Player.clj`: `(ns Solution(:gen-class))` fails at compile with `Exception: namespace 'Player' not found after loading '/Player'`, turn 1 times out, 0%. Write `(ns Player(:gen-class))`
- the interactive runner calls `Player/-main` with one argument (`nil`): `(defn -main[]...)` dies with `ArityException: Wrong number of args (1) passed to: Player/-main`. Use `(defn -main[_]...)` (verified tests 2 and 4 pass) or `[& _]`
- runtime there: `sun.java.command` = `Bridge`, `java.version` 1.8.0_211 (not the Java 21 of io puzzles): no Java 9+ APIs (`String/repeat`, `.lines`, `.strip`) on interactive puzzles
- timing: JVM uptime at `-main` entry 1448 ms, first answer 1619 ms, still passed the 1000 ms first turn: the turn timer starts after the bridge is up, Clojure is not too slow for power-of-thor
- local trap: `agent/tool/lang/clojure/build.sh` hardcodes `Solution.clj` and zero-arg main, so the local referee passes `ns Solution`+`-main[]` (CG 0%) and fails `ns Player`+`-main[_]` (CG pass) until the launcher follows `main_class` like java/scala
