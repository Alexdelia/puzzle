# Interactive: print every turn's answer up front, then exit

CG's interactive referee keeps reading the player's buffered stdout after the player process exits: a failed write to the dead player's stdin is ignored, and the game continues turn by turn while lines remain. No per-turn read loop and no keepalive are needed.

- verified CG: the-bridge-episode-2 ruby 152 B prints all turns on turn 1 and exits (`puts ...`), 100% and rank 1 (2026-10-24); python/perl plan-once rows also 100%
- golf: whole-plan solvers can end with `exit(print(q))` / `puts ...` with no `$|` and no `sleep`
- local: shared `referee-core` `Interactive::play` fails on the first failed write ("solution closed its input"); the player must stay alive for a local pass (e.g. keepalive `;sleep` in Perl), and that race is load-dependent. the-bridge-episode-2, the-fall-episode-2 and vox-codei-episode-2 got a puzzle-local `ClosedStdin` runner on 2026-10-25 that ignores failed writes like CG; other puzzles still use the core loop
- `submit.sh --probe` skips the local `--cg` gate, but a local reject on the real row blocks submits: fix the puzzle referee (puzzle scope) or add the keepalive
- not verified: whether a player that exits *without* enough buffered lines is judged the same on every interactive puzzle; CG likely fails it when the reader runs dry

- vox-codei-episode-2 check: 4 core langs 3 runs each under 8 CPU hogs -> all 10/10 hidden; a player exiting with one line only still fails `solution exited`
