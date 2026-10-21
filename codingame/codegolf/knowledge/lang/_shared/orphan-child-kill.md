# Orphan child killed after main exit

Verified 2026-10-21 on CG (the-labyrinth test 4, Groovy, test games in `agent/state/journal/referee.the-labyrinth.-.1792607002123773968.md`): a child spawned by the solution (`ProcessBuilder(...).inheritIO().start()` without `.waitFor()`) keeps playing after the main process exits, but CG kills it about 0.3-0.5 s after that exit.

- main = JVM exiting right after `start()`, perl child sleeping 0.1 s per turn: answered at t=0.10, 0.21 s, killed before 0.31 s (frame 3 timeout)
- main sleeping 3 s before exit (`jvm exit 3.01` on stderr), same child: answered up to t=3.43 s, timeout at the next turn (~3.53 s)
- main exiting at once, fast child (~5 ms/turn): 59 moves, timeout frame 60

Measured locally 2026-10-21: a JVM whose child is still alive takes ~0.4 s to exit after its main returns (0.1 s without a child). So most of CG's 0.3-0.5 s is JVM shutdown: CG kills the children at, or within ~0.1 s of, the real exit of the main process.

Interactive only: io puzzles keep the child (mayan-calculation and music-scores groovy without `.waitFor()` CG 100%).

So "no wait" wrappers (`.start()`, `exec` without wait, background `&`) only work when the whole game or io answer fits in ~0.3 s after main exit. Interactive games running longer need the wait (`.waitFor()` in Groovy, +10 B), or an `exec` that replaces the main process.

Local referee-core models it since 2026-10-21 (interactive only): the process group is killed 50 ms after the launcher exits (`agent/tool/referee-core/src/process.rs`). the-labyrinth groovy 257 now fails locally (5/8 tests) as on CG.
