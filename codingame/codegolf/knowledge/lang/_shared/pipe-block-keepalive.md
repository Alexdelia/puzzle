# Keep an interactive bot alive by blocking on a full stdout pipe

Open-loop interactive answers (whole plan printed at turn 1): exiting right after printing used to be racy locally ("solution closed its input"), and reading stdin to EOF costs bytes (Perl `;()=<>` 6, Lua `io.read"a"`). CG itself tolerates the exit and reads the buffered plan (see [interactive-early-exit](interactive-early-exit.md)); the shared referee-core loop does not, so this trick still buys a clean local pass on puzzles whose referee was not fixed.
Instead make the last segment huge: repeat the last answer `1e5` times (400 KB > 64 KB pipe buffer). The write blocks because the referee reads one line per turn, the process stays alive until the game ends.

- Perl: `print"$1\n"x$2while/(.+) (.+)/g` with the last count `1e5` (numifies to 100000) replaces `;()=<>`: Mars Lander 3 perl 95 -> 92, CG 100% (probe 2026-10-21, probes.tsv), local referee 3/3
- the repeated last answer must be harmless after landing (game is over by then)
- the string is built in memory: keep it to a few MB (1e5-1e6 lines)
