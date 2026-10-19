# Kotlin runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19, probes print to stderr).

- Kotlin 1.7.10 (`KotlinVersion.CURRENT`), run on Java HotSpot 21.0.4+8-LTS-274 with `-Xverify:none` (JVM prints a deprecation warning to stderr, harmless)
- but kotlinc compiles against a Java 8 JDK: Java 9+ APIs do not resolve (`Runtime.version()` -> `error: unresolved reference: version`, compile error, test fails). Only Java 8 APIs plus the Kotlin 1.7 stdlib are usable, even though the runtime is 21
- Kotlin 1.7 stdlib: `readln()`, `readlnOrNull()`, `Char.code`, `String.repeat`, `lines()` (inferred from the stdlib version, only `Char.code`, `String.repeat` and `readLine()!!` run on CG)
- local: `agent/tool/lang/kotlin/build.sh` compiles with kotlinc 1.7.10 on OpenJDK 8u504 (same unresolved-reference error for Java 9+ APIs, verified) and runs on OpenJDK 21.0.12 with `-Xverify:none`, classpath kotlin-stdlib(+jdk7, jdk8) + kotlin-reflect, main class `SolutionKt`
- drift: OpenJDK vs Oracle HotSpot, patch 21.0.12 vs 21.0.4, CG main class / classpath / heap: see probe line below
- launcher startup ~0.1-0.3 s warm (smoke program), counts in the 6 s io limit
