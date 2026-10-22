# Bare toplevel expression

Verified 2026-10-22 (shadows-of-the-knight-episode-1, CG 100%): a file that is only `Sys.command"bash -c '<core>'"` compiles and runs (int-typed toplevel expression, no `let _=` or `;;`). Shell wrapper overhead 23 (`Sys.command` runs `/bin/sh` = dash, hence `bash -c`).
