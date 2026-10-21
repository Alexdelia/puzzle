# Groovy: first turn of interactive puzzles is borderline on CG (verified 2026-10-21, dont-panic)

The local referee does NOT model it: a Groovy solution passing locally can time out at frame 1 on CG ("Timeout: the program did not provide 1 input lines in due time"), randomly per run. Explains dont-panic groovy 85.7 and probably power-of-thor groovy 50%.

Measured on CG test games (stderr `java.lang.management.ManagementFactory.runtimeMXBean.uptime`, dont-panic, games 910612460-910613300):

- script `main` reached at JVM uptime 1175-1334 ms on CG (local 623-645 ms: CG JVM start ~2x slower)
- first answer deadline ~1490-1520 ms uptime: busy-wait to 1490 pass, 1519 pass, 1500/1530/1550/1557/1600 timeout. CG timer starts ~500 ms into JVM life (first-turn limit 1000 ms)
- so `main` -> first answer must take <= ~170-320 ms on CG, about <= 90 ms local (main -> answer ratio CG/local ~2.4)
- native dont-panic 137 (`System.in.eachLine{...tokenize()...}`, map, ternaries): main -> answer 100-113 ms local, 240-280 ms CG: 4/7 IDE tests timed out in one batch, 6/7 validators twice

Check locally before submitting an interactive Groovy cell: print `runtimeMXBean.uptime` to stderr at script start and before the first `println`; local difference must stay well under ~90 ms (first-use cost of closures, dynamic dispatch, `tokenize`, `as int` class loading counts). Safer: hand the game to a fast language via `new ProcessBuilder(...).inheritIO().start()` (see [perl-wrapper-processbuilder](perl-wrapper-processbuilder.md); shadows-of-the-knight groovy 141 bash wrapper passed CG), length permitting.

Next turns (150 ms) are not affected once warm (dont-panic games that survived frame 1 won with the local turn counts).

Data point the-descent (2026-10-21, games 910617668-910617754): `for(;;)println((0..15).max{System.in.read()}>>1)` local main -> answer 79-107 ms; CG main at 1162-1326 ms, answer at 1437 passed, 2/3 instrumented games timed out at frame 1, plain code 5/5 tests then 79.976 on submit (1 validator out). Range `max` with a closure is not cheap enough on a cold JVM.
