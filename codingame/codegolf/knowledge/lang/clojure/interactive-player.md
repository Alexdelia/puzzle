# Clojure on interactive puzzles: `ns Player`, `-main` takes 1 arg

CG verified 2026-10-21 (power-of-thor test games 910670828, 910670850, 910671093, 910671218, referee.power-of-thor journal 1792607297208713249).

- interactive puzzles compile the file as `Player.clj`: `(ns Solution(:gen-class))` fails at compile with `Exception: namespace 'Player' not found after loading '/Player'`, turn 1 times out, 0%. Write `(ns Player)`: no `:gen-class` needed, the bridge resolves the `Player/-main` var (CG games 910670910, 910672642 with plain `(ns Player)`)
- the interactive runner calls `Player/-main` with one argument (`nil`): `(defn -main[]...)` dies with `ArityException: Wrong number of args (1) passed to: Player/-main`. Use `(defn -main[_]...)` (verified tests 2 and 4 pass) or `[& _]`
- runtime there: `sun.java.command` = `Bridge`, `java.version` 1.8.0_211 (not the Java 21 of io puzzles): no Java 9+ APIs (`String/repeat`, `.lines`, `.strip`) on interactive puzzles
- timing: JVM uptime at `-main` entry 1448 ms, first answer 1619 ms, still passed the 1000 ms first turn: the turn timer starts after the bridge is up, Clojure is not too slow for power-of-thor
- local launcher follows this since lang.-.clojure.1792607556179713326: interactive builds compile `Player.clj` (`clojure.lang.Compile Player`, Java 8) and run a `Bridge` class on OpenJDK 8 that requires `Player` and invokes `-main` with one nil: local `sun.java.command` = `Bridge`, `-main[]` dies with the same ArityException, `ns Solution` fails at compile like CG
