# `io.Source` does not compile on CG: write `scala.io`

CG verified (ascii-art-codesize test game, 2026-10-21): `io.Source.stdin` fails to compile:

```
/tmp/Solution.scala:1: error: object Source is not a member of package io
```

Cause: CG compile classpath holds a root package `io` that shadows `scala.io` (root-package members beat the `scala._` wildcard import).

- `/codemachine/lib/scala/external/` on CG holds ~150 jars (Spark 2.13, circe, netty, cats, shapeless, json4s, jackson, guava, scalatest...): listed by a probe (journal `referee.ascii-art.-.1792593250096155138`)
- root packages of those jars: `cats com io javassist javax jersey net org py4j scala shaded shapeless`. Only `io` collides with a `scala._` member (`io.circe`, `io.netty`...)
- inferred, not probed directly: scalac compile classpath includes `external/*` (the error needs a root `io`, and nothing else on CG provides one). Run classpath is `/tmp` only (`Tester.sh`: `scala -cp $CLASSPATH Solution`, `CLASSPATH=/tmp`; `external/*` added only when `CODE_TYPE=CODEAPI_EXECUTE`, unset in test games): library classes from those jars may compile but fail at run time (unverified)
- consequence: every `io.X` short form (`io.Source`, `io.StdIn`) breaks; write `scala.io.Source` / `scala.io.StdIn` (+6) or `import scala.io._`
- local toolchain reproduces it since lang task `lang.-.scala.1792593505404767254` (2026-10-21): `scalac` launcher has a stub `io.circe.Stub` on its classpath, `io.Source` / `io.StdIn` fail locally with the CG error text. Older local builds (before that package) accepted `io.X`: re-check any scala cell measured before
