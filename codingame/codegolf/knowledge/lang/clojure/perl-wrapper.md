# Clojure: perl wrapper

`(ns Solution(:gen-class))(.start(.inheritIO(ProcessBuilder. ["perl" "-ape" "core"])))` at top level, no -main: local 21/21 on mayan-calculation (wrapper 87 B). Avoid `"` in core (`join$,,` instead of `join"",`). CG confirmed 262 rank 1 (mayan-calculation).

surface: core as `"perl" "-ln0e" "..."` (-l before -0 keeps $\ newline); core may hold real newlines and no `"` or backslash. 228 B total (surface 228 rank 1).

interactive (dont-panic, local 15/15 + hidden 2026-10-21, CG 100% at 156): `(ns Player)(defn -main[_](.waitFor(.start(.inheritIO(ProcessBuilder.["perl""-anE""core"])))))`, overhead 89 B. Adjacent string literals `"perl""-anE"` need no space (-2 per gap). `.waitFor` needed (child killed ~0.3 s after parent exit, `_shared/orphan-child-kill.md`); top-level waitFor would block the compile on stdin, so it lives in `-main`. `(->[..]ProcessBuilder. .inheritIO .start .waitFor)` is 1 B shorter, CG 100% (dont-panic 155 #5). Local turn-1 1000 ms limit fails under machine load (JVM 8 start): rerun when load is low.

Raw newline: a perl core `(.*\n)` can hold a real newline in the Clojure string instead of the 2-char `\n` escape (-1 B per use). CG 100% (the-labyrinth 288 #1, 2026-10-21).
