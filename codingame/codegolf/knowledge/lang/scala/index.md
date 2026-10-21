# Scala

- [env](env.md): 2.13.5, compiled on JDK 16-20 (local 17), run via `scala` runner on Java 21, main object must be `Solution`, no script mode; verified
- [root io package](root-io-package.md): `io.Source` fails on CG (root package `io` from external jars shadows `scala.io`), use `scala.io.Source`; local build reproduces it (stub on scalac classpath); verified
- [io-shadowed-console-in](io-shadowed-console-in.md): `io.`/`util.` shadowed on CG (use `Console.in.lines.toArray.map(_+"")`); `c-65&223 min 26` letter index clamp
