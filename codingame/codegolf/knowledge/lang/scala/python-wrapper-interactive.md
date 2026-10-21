# Scala: run a python core from an interactive puzzle

Verified 2026-10-21 (the-fall-episode-2, CG 100%, scala row 1123, rank 6): `object Player extends App{new ProcessBuilder("/usr/bin/python3","-c","""<python>""").inheritIO.start.waitFor}` = python core + 100 bytes. Python core must not contain `"""`.

- `sys.process.Process(...).!` gave 0%: `!` does not connect stdin (`!<` would); `object Solution` also gives 0% on interactive puzzles (`main-object-player.md`). The ProcessBuilder form inherits stdin/stdout and answers every turn
- `/usr/bin/python3` exists in the Scala box (see `../_shared/system-wrapper.md`)
- local `submit.sh` cannot run it (no python3, `Player`): send through `oracle.sh` from `probe/scala/`
