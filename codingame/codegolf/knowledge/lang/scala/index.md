# Scala

- [env](env.md): 2.13.5, compiled on JDK 16-20 (local 17), run via `scala` runner on Java 21, main object must be `Solution` (io), `Player` on interactive puzzles, no script mode; verified
- [root io package](root-io-package.md): `io.Source` fails on CG (root package `io` from external jars shadows `scala.io`), use `scala.io.Source`; local build reproduces it (stub on scalac classpath); verified
- [io-shadowed-console-in](io-shadowed-console-in.md): `io.`/`util.` shadowed on CG (use `Console.in.lines.toArray.map(_+"")`); `c-65&223 min 26` letter index clamp
- [console-in-lines](console-in-lines.md): `io.Source` breaks on CG (0%), use `Console.in.lines` stream + `reduce`; `tails.filter.toSeq.last`; verified
- [io-package-shadowed](io-package-shadowed.md): `io.StdIn` fails on CG (local ok), use `Console.in.readLine`; String.flatMap returns String
- [read-all-bytes](read-all-bytes.md): `System.in.readAllBytes` = whole stdin 21 chars (vs scala.io.Source 36), numeric char codes, tuple vars, `0 until 4` reversible; verified
- [main-object-player](main-object-player.md): interactive puzzles run `Player`, not `Solution` (`ClassNotFoundException: Player$`); local launcher only knows `Solution`; verified
- [scanner-unzip-doubled-median](scanner-unzip-doubled-median.md): Scanner+Seq.fill+unzip input, (y++y).sorted.splitAt median sum; List index O(n) trap; verified
- [interactive-byte-argmax](interactive-byte-argmax.md): `maxBy(_=>System.in.read)>>1` over 0..15 for line-per-height interactive puzzles; Player vs Solution local caveat; verified
- [set-frontier-flood](set-frontier-flood.md): immutable Set BFS with persistent reset per query, regex extractor in for; surface 326
- [python-wrapper-interactive](python-wrapper-interactive.md): `object Player` + `new ProcessBuilder("/usr/bin/python3","-c","""..""").inheritIO.start.waitFor`, core+100 B; `sys.process !` has no stdin; verified
