# Scala: sys.process pipeline with stdin, no sh

Verified 2026-10-21 (super-computer, local 15/15 + CG submit 100%, scala row 103, rank 2):

- `object Solution extends App{import sys.process._;("sort -nr"#|"awk PROG"#|"wc -l").!<}`: `!<` runs the builder with the JVM stdin connected to the first process, output copied to stdout, blocks until exit
- String commands are split on whitespace: an awk program with no spaces and no quotes passes as one argument, no `sh -c` and no shell quoting needed; `$1` is literal in a plain Scala string
- chain overhead: header + import 49, `(`...`).!<}` 5, `#|` 2 per stage. Shorter than `new ProcessBuilder("sh","-c","core").inheritIO.start.waitFor` (+16 bytes here)
- postfix `chain!<` (no parens, no dot) = compile error in 2.13.5 (`postfix operator !< needs to be enabled`)
- `#|` has higher precedence than alphanumeric `run`; `run(true)` infix (`"x"run 1>0`) is longer than `().!<` anyway
- unverified: interactive puzzles (output flushing through the copy thread), `Seq("python3","-c","""...""").!<` as a replacement for the ProcessBuilder wrapper of `python-wrapper-interactive.md` (would save ~15 bytes)
