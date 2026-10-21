# `io.` and `util.` are shadowed on CG; read stdin with `Console.in.lines`

Verified on CG test games (2026-10-21, ascii-art): `io.Source.stdin` fails to compile (`object Source is not a member of package io`), also `io.StdIn`, `util.Scanner`: `io` and `util` resolve to empty packages (likely `scala.tools.nsc._` in the CG prelude). Local run.sh does NOT reproduce it: always test a Scala solution with one CG test game before submitting.

- `scala.io.Source.stdin.getLines.toSeq`: works, 36 bytes
- `Console.in.lines.toArray.map(_+"")`: works (BufferedReader.lines is a Java stream, `toArray` gives `Array[Object]`, `_+""` makes them `Array[String]`), 34 bytes
- `Console.in.lines.toList` (Java 16, typed `java.util.List[String]`) is no shorter overall: no `drop`/`apply`

Unsigned-free clamp of a char to a 0..25 letter index, else >=26: `c-65&223 min 26` (`-` binds tighter than `&`, `min` is lowest; `&223` clears the lowercase bit and turns negatives into >=64). Checked on all chars 32..126. Used as `r.grouped(n).toSeq(c-65&223 min 26)`.
