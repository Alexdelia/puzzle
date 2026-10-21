# Clojure: perl wrapper

`(ns Solution(:gen-class))(.start(.inheritIO(ProcessBuilder. ["perl" "-ape" "core"])))` at top level, no -main: local 21/21 on mayan-calculation (wrapper 87 B). Avoid `"` in core (`join$,,` instead of `join"",`). CG confirmed 262 rank 1 (mayan-calculation).

surface: core as `"perl" "-ln0e" "..."` (-l before -0 keeps $\ newline); core may hold real newlines and no `"` or backslash. 228 B total (surface 228 rank 1).

interactive (dont-panic, local 15/15 + hidden 2026-10-21, CG 100% at 156): `(ns Player)(defn -main[_](.waitFor(.start(.inheritIO(ProcessBuilder.["perl""-anE""core"])))))`, overhead 89 B. Adjacent string literals `"perl""-anE"` need no space (-2 per gap). `.waitFor` needed (child killed ~0.3 s after parent exit, `_shared/orphan-child-kill.md`); top-level waitFor would block the compile on stdin, so it lives in `-main`. `(->[..]ProcessBuilder. .inheritIO .start .waitFor)` is 1 B shorter, CG 100% (dont-panic 155 #5). Local turn-1 1000 ms limit fails under machine load (JVM 8 start): rerun when load is low.

Raw newline: a perl core `(.*\n)` can hold a real newline in the Clojure string instead of the 2-char `\n` escape (-1 B per use). CG 100% (the-labyrinth 288 #1, 2026-10-21).

Overhead with `..` (`dotdot-chain.md`), review 2026-10-21: `(ns Solution(:gen-class))(..(ProcessBuilder.["perl""-e""CORE"])inheritIO start)` = 75 B + core (+1 per extra flag letter); interactive `(ns Player)(defn -main[_](..(ProcessBuilder.["perl""-anE""CORE"])inheritIO start waitFor))` = 85 B + core with `-e`. With 3 calls `..` beats `(->[..]ProcessBuilder. .inheritIO .start .waitFor)` by 2 B (dont-panic 155 -> 153, the-labyrinth 288 -> 286 local).

Rule of thumb: compare every native clojure cell with the shortest perl core of the puzzle (`puzzle/<p>/solution/perl`, or the perl inside `solution/bash`) + 75/85. Local referee pass 2026-10-21: ascii-art 184 -> 138, chuck-norris 177 -> 133, blunder-episode-1 409 -> 342, roller-coaster 221 -> 211, blunder-episode-3 308 -> 250 (see journal of review.-.clojure for CG results). Quoting: each `"` or `\` in the core costs 1 B (`\"`, `\\`); switch perl strings without interpolation to `'..'` (free inside a Clojure string), `qq(..)` when interpolating, raw newline for `\n`. A `-0p`/`-nl` shebang flag becomes the `-0pe`/`-nle` arg. Perl cores get no "-l" for free: count the flag letters.
