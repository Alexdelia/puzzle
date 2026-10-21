# Orphan child killed after main exit

Verified 2026-10-21 on CG (the-labyrinth test 4, Groovy, test games in `agent/state/journal/referee.the-labyrinth.-.1792607002123773968.md`): a child spawned by the solution (`ProcessBuilder(...).inheritIO().start()` without `.waitFor()`) keeps playing after the main process exits, but CG kills it about 0.3-0.5 s after that exit.

- main = JVM exiting right after `start()`, perl child sleeping 0.1 s per turn: answered at t=0.10, 0.21 s, killed before 0.31 s (frame 3 timeout)
- main sleeping 3 s before exit (`jvm exit 3.01` on stderr), same child: answered up to t=3.43 s, timeout at the next turn (~3.53 s)
- main exiting at once, fast child (~5 ms/turn): 59 moves, timeout frame 60

So "no wait" wrappers (`.start()`, `exec` without wait, background `&`) only work when the whole game or io answer fits in ~0.3 s after main exit. Interactive games running longer need the wait (`.waitFor()` in Groovy, +10 B), or an `exec` that replaces the main process.

Local referee-core does NOT model it (proposal in the journal above): the local referee passes such code (the-labyrinth groovy 257 local 8/8, CG 25%). Until it does, check by hand that the main process outlives the game.
