# Clojure: spawn wrapper on interactive puzzles

Verified on CG 2026-10-21 (shadows-of-the-knight-episode-2, journal cell.shadows-of-the-knight-episode-2.clojure.1792513328992611527).

- the spawned core must start inside `-main`: `(ns Player)(defn -main[_](..(ProcessBuilder.["bash""-c""<core>"])inheritIO start waitFor))` won test 7 (test game 910675811)
- top-level spawn with `waitFor` (no `-main`) fails on CG: the compile JVM's stdin is an open pipe there, the child blocks reading it, "Compilation took too long and has been interrupted" (test game 910675664). Locally the compile stdin is a directory, so the child exits at once and the local run passes: local referee does not catch this
- local referee counts JVM+Clojure startup in the 1000 ms first turn (CG does not): under host load the whole local suite times out at turn 1. Not a CG failure; send such files through `probe/clojure/` + `oracle.sh`
- a dev shell may carry a stale `CG_CLOJURE` (compile script hardcodes `Solution`, no `clojure-run-interactive`): export the newest `/nix/store/*-cg-clojure-1.11.1` that has `bin/clojure-run-interactive`
- 2026-10-21: of the store builds carrying `clojure-run-interactive`, some pass `Bridge "$2"` (class name) which referee.sh does not give: `ClassNotFoundException: ` on every case, "solution exited". The one without `"$2"` works (`/nix/store/rrydskrr4gpyrf03qwdqn90vy8jxjxj5-cg-clojure-1.11.1` at that date): check `cat $CG_CLOJURE/bin/clojure-run-interactive`
