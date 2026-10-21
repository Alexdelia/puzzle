# Groovy runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- `GroovySystem.version` 3.0.8 on CG and locally (apache-groovy-binary-3.0.8 zip)
- CG JVM: `Java HotSpot(TM) 64-Bit Server VM 25.211-b12`, `java.version` 1.8.0_211; local: OpenJDK 8u504 (nixpkgs `jdk8`). Same Java 8 API
- script class is `Answer` (`this.class.name`), precompiled: `sun.java.command=Answer`, `args` empty, `file.encoding` UTF-8, `groovy.target.indy` unset
- local `agent/tool/lang/groovy/build.sh`: `groovyc` to `Answer.class`, launcher `java -cp classes:lib/* Answer`, mirrors CG
- `assert` cannot hold an assignment (`assert x = true` is a compile error), Groovy asserts are always on
- stderr is shown in the test-game `output` but not judged
- startup: local launcher ~0.65 s wall on an idle machine (2-3 s under load); count it against turn limits
- CG startup is ~0.75 s slower than local (dont-panic test games 2026-10-21, `ManagementFactory.runtimeMXBean.uptime` at first answer): minimal reader loop 1360 ms on CG vs 608 local; `System.in.eachLine{(f,p,d,E,P)=it.tokenize()...}` map solution 1523-1530 on CG vs 772-803 local. Interactive puzzles with a 1000 ms first turn time out at frame 1 when uptime reaches ~1.53 s: local referee passes, CG flaky (dont-panic groovy 137: 85.7). Keep first-turn code paths minimal, CG-test before submit
