# Scala: main object is Player on interactive puzzles

Verified 2026-10-21 (shadows-of-the-knight-episode-1, interactive, CG test games 910623693 and two more):

- `object Solution extends App{...}` fails turn 1: `error: java.lang.ClassNotFoundException: Player$`, timeout, submit 0%
- same code with `object Player` wins tests 4 (1x80 N6) and 7 (9999x9999 N14), first turn in time even with a `sys.process` bash child
- io puzzles keep `Solution` (`env.md`); Java has the same split (`../java/main-class.md`)
- local launcher `agent/tool/lang/scala/build.sh` always runs `Solution`: the local referee passes `Solution` and fails `Player` on interactive puzzles, the opposite of CG. Until it learns the puzzle kind, interactive Scala cells cannot go through `submit.sh`
- workaround (the-fall-episode-3, 2026-10-21): put the `object Player` file in `probe/scala/` and send it with `oracle.sh`; at 100% and shorter than the row it is committed as the solution

Fixed 2026-10-21: local scala and java launchers run `Player` on interactive puzzles and `Solution` on io (`CG_PUZZLE_KIND` from `referee.sh`), so a wrong main object fails locally like on CG.
