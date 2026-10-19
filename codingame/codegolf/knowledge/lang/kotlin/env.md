# Kotlin runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19, probes print to stderr).

- Kotlin 1.7.10 (`KotlinVersion.CURRENT`, stdlib manifest `1.7.10-release-333`), run on Java HotSpot 21.0.4+8-LTS-274
- run command: `java -Xms64m -Xmx450m -Xverify:none -Djava.awt.headless=true -Djava.io.tmpdir=/tmp -ea -cp <jars>:/tmp AnswerKt`, cwd `/tmp`: source file is `Answer.kt`, assertions enabled (`-ea`), JVM prints an `-Xverify:none` deprecation warning to stderr (harmless)
- classpath jars (`/opt/coderunner/kotlin/lib/`): kotlin-stdlib-jdk8, kotlin-stdlib-jdk7, kotlin-stdlib, annotations, kotlinx-coroutines-core (version not in manifest, local assumes 1.6.4, unverified), kotlinx-serialization-core-jvm 1.4.0 (manifest), kotlinx-serialization-json-jvm. No kotlin-reflect
- kotlinc compiles against a Java 8 JDK: Java 9+ APIs do not resolve (`Runtime.version()` -> `error: unresolved reference: version`, test fails). Only Java 8 APIs plus the Kotlin 1.7 stdlib are usable, even though the runtime is 21
- Kotlin 1.7 stdlib: `readln()`, `readlnOrNull()`, `Char.code`, `String.repeat`, `lines()` (inferred from the stdlib version, only `Char.code`, `String.repeat` and `readLine()!!` run on CG)
- local: `agent/tool/lang/kotlin/build.sh` compiles `Answer.kt` with kotlinc 1.7.10 on OpenJDK 8u504 (same unresolved-reference error for Java 9+ APIs, verified) and runs OpenJDK 21.0.12 with the CG flags, jar names and order
- drift: OpenJDK vs Oracle HotSpot, patch 21.0.12 vs 21.0.4, coroutines version guessed
- launcher startup ~0.07 s warm, ~0.2-0.3 s first run (smoke program), counts in the 6 s io limit
