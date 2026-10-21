# `io.StdIn` does not compile on CG

Verified by CG test game (chuck-norris, 2026-10-21): `io.StdIn.readLine` -> `error: object StdIn is not a member of package io` (the `io` shorthand for `scala.io` resolves to another package there); local scalac accepts it, so the local referee passes and CG scores 0.0.
Works on CG and locally: `Console.in.readLine` (+2 over `io.StdIn.readLine`), `scala.io.StdIn.readLine` (+6). `readLine` alone does not exist in 2.13.
`String.flatMap(c=>"str")` returns a String (no `.mkString` needed): saves 5 vs `map(..).mkString`.
When local passes but submit.sh row shows score 0.0: run one test game with `agent/tool/cg.sh POST /puzzles/<id>/test-games` to read the compile error.
