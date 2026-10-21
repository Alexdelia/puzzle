# Scala runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19, probes print to stderr).

- Scala 2.13.5 (`scala.util.Properties.versionNumberString`), run on Java HotSpot 21.0.4+8-LTS-274
- source saved as `/tmp/Solution.scala`, compiled with scalac, run `scala -cp /tmp Solution` (`scala.tools.nsc.MainGenericRunner`), cwd `/tmp`
- main object must be `Solution` on io puzzles (`Player` on interactive ones, `main-object-player.md`): `object A extends App` -> `No such file or class on classpath: Solution`. Top-level statements (script mode) -> compile error `expected class or object definition`
- JVM args: `-Xmx256M -Xms32M`, bootclasspath = jline-3.19.0, jna-5.3.1, scala-compiler, scala-library, scalap-2.13.5, scala-reflect: compiler and reflect classes are reachable at run time; 1 CPU, max heap 259522560, UTF-8
- compile JDK is 16-20 (not 8 like Kotlin, not 21): `Runtime.version`, `List.of`, `"".strip`, `describeConstable`, `java.lang.Record` resolve, `java.util.SequencedCollection` does not. Java 21-only API unusable even though runtime is 21
- local `agent/tool/lang/scala/build.sh`: scalac 2.13.5 (Maven Central jars) on OpenJDK 17, run with the dist `scala` launcher flags on OpenJDK 21 plus `-XX:ActiveProcessorCount=1`; same errors as CG on all probes above
- scalac 2.13.5 on JDK 21 crashes (`bad constant pool index: 0`): never compile on 21
- drift: OpenJDK vs Oracle HotSpot, patch 21.0.12 vs 21.0.4, compile JDK 17 assumed within 16-20
- startup: launcher ~0.5 s wall warm (smoke), up to ~1 s; counts in the 6 s io limit
