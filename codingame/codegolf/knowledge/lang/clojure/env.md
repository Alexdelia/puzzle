# Clojure runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19, probes print to stderr).

- two JVMs: compile `java -Xmx450m -Xverify:none -Dclojure.compile.path=/tmp -cp clojure.jar:spec.alpha.jar:core.specs.alpha.jar:/tmp/deps/*:/tmp clojure.lang.Compile Solution` on Oracle Java 1.8.0_211, then run `java -cp <same jars>:/tmp/ Solution` on Java HotSpot 21.0.4 (no JVM flags, maxMemory 194641920, cwd `/tmp`)
- Clojure 1.11.1. Help center "Oracle JVM 1.8" is only the compile JVM: runtime Java APIs are Java 21 (`String/repeat`, `.lines`, `.strip`, ... inferred available, not run on CG)
- code is AOT compiled as `Solution.clj`: needs `(ns Solution(:gen-class))`, otherwise no `Solution` class, compile reported as error
- top-level forms run twice: at compile (`*compile-files*` true, output hidden, not judged) and at run when the class loads (`*compile-files*` false)
- `-main` can be left out: top-level output is judged, then `UnsupportedOperationException: Solution/-main not defined` (crash after output passes, see `_shared/crash-exit.md`) - verified the print part, the full pass not yet
- but reading stdin at top level fails the compile (`(read-line)` at top level of a `:gen-class` file: error `Compiling Solution to /tmp`, nothing else): top-level stdin reads must be guarded by `*compile-files*` or live in `-main`
- stdout at compile is harmless (verified with a top-level `println`)
- local: `agent/tool/lang/clojure/build.sh` does the same two steps (OpenJDK 8u504 compile, OpenJDK 21.0.12 run, same jar versions), compile stdin is a directory so a top-level read fails the build like on CG
- drift: OpenJDK vs Oracle/HotSpot, patch 21.0.12 vs 21.0.4, local heap not capped to ~186 MB
- launcher startup ~0.6 s warm, ~1.1 s cold (JVM + clojure core load), counts in the 6 s io limit
