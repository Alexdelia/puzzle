# Groovy io time limit on CG (verified 2026-10-21, super-computer-codesize test games)

The io limit is NOT the 6000 ms of `lang/_shared/io-time-limit.md` for Groovy. Probe: busy-sleep until JVM uptime T (`ManagementFactory.runtimeMXBean.uptime`), then print the answer:

- T=1400, 1700, 1825 (x2) on time; T=1950, 2200, 3000, 5500 timeout. Deadline ~1.85-1.95 s of JVM uptime, script body starts at uptime 944-1138 ms: about 0.8 s left for user code
- same puzzle, Python3 `time.sleep(3)` then answer: on time. The limit is per language
- CG runs user code ~2-3x slower than local: reading 1 MB stdin (`System.in.text`) took 324 ms on CG vs 110-190 ms local
- CG max heap (`Runtime.maxMemory`) 290 MB
- consequence: native Groovy over large input times out. super-computer (N=99999, 1 MB): `System.in.text.split()*.toInteger()...sort{}` and `readLines()...collect{split()}...sort{}` both 15/15 local (1.2-1.9 s wall), both 87.5 on CG (one large validator timed out); test 03 probe timed out during the parse
- way out: shell/perl wrapper, `new ProcessBuilder("sh","-c",'<core>').inheritIO().start()` read 1 MB and answered on CG test 03 at uptime ~1.1 s (`[[perl-wrapper-processbuilder]]`)
- local referee uses 6000 ms for every language and local JVM starts 2x faster: a native Groovy solution on large input passing locally proves nothing. CG-test the largest IDE test before submit
- unverified: other JVM langs (java, kotlin, scala, clojure) may have the same tight limit
- second timing point (referee journal 1792592463285021254): `readLines()` variant on test 03, CG start 969, read +378 ms (local +150), `drop(1).collect{it.split()*.toInteger()}` +472 ms (local +170), timeout before the sort ends: CG ~2.5x local. Rough local bar, unverified: whole run under ~0.9 s wall on an idle machine
- third puzzle, the-resistance-codesize test 01 (referee journal 1792592174068154900): busy-wait to T=1600, 1700, 1800, 1900 (end uptime 1926) on time; T=2000, 3000, 5500 timeout; `Thread.sleep` to 3000/5500 timeout too (wall clock, not CPU). Same T=3000 on chuck-norris-codesize: timeout. Deadline ~1.93-1.95 s uptime, every io puzzle. Python sleep 5.5, bash sleep 2.5, JS busy-wait 5.0 on time: Groovy only (among those)
- calibration there (machine load ~5): honest DP end uptime CG vs local, t01 1397/830, t02 1446/840, t03 1603/835, t05 1200/890, t06 1455/850, t04 timeout/1580 (local wall = uptime + 50-90 ms). Model CG_up ~ 900 + 2 x (local_up - 500): local wall bar ~1.08 s under load
- the-resistance referee applies a Groovy limit of 1100 ms wall (launcher path `.../lang/groovy/...`), 6000 ms for others; other referees still use 6000 for Groovy
