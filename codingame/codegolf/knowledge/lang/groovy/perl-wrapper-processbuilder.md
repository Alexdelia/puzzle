# Groovy perl wrapper (mayan-calculation, CG accepted 237 #1, 2026-10-21)

`new ProcessBuilder("perl","-ape",'<code>').inheritIO().start()` - perl is in the Groovy CG container. No `sh -c` quoting needed, no `.waitFor()` needed (CG captured the output anyway, -10 B). Perl core: see knowledge/puzzle/mayan-calculation.md.
- native Groovy was 318+: `Eval.xy(x,y,"x${op}y")` avoids Integer overflow (needs Long inputs), BigDecimal from `/` breaks `%`
- TRAP long interactive games (the-labyrinth, CG 2026-10-21): without `.waitFor()` the child dies mid-game once the JVM main has exited: test 4 timed out at frame 60 (test 1, 14 frames, passed), submit 25%. `.start().waitFor()` (+10 B) passed test 4 (208 frames) and the submit 100%. Short games (shadows, the-fall, the-bridge) passed without it; add it whenever a game can run long
