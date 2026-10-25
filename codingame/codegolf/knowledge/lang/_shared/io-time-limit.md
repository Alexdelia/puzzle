# io time limit

Probe 2026-10-19 on `chuck-norris-codesize` test 1, Python `time.sleep(s)` then solution:

- 5.95 s pass, 6 s timeout
- limit about 6000 ms per test, interpreter startup included
- referee uses 6000 ms for every io puzzle unless a puzzle note says otherwise
- per language, not global (verified 2026-10-21, super-computer-codesize): Groovy deadline ~1.9 s of JVM uptime, Python3 `sleep(3)` on time on the same test. See `lang/groovy/io-time-limit.md`. Other languages not probed: the 6000 ms is verified for Python only

## Test games are NOT a validator proxy (verified roller-coaster 2026-10-25)

`puzzle/roller-coaster-codesize` testcase 6 (L=1e7 C=9e6): the known-100% C solution
`solution/c/main.c` (164 B, method B in spawned perl, ~0.65 s local) answers **timeout** in a test
game, while the Perl 105 solution (pure perl method C) passes the same test game and both are 100%
on validators. So a test-game timeout does not mean a validator timeout; judge runtime by submissions
or by a language whose row is not 100%.

Language limits inferred from rows/wraps on this puzzle (idle-local runtime, unverified directly):
C validator limit sits between method B (~0.65 s) and method C (~1.4-2 s) in a spawned perl; D is
similar (wait-spawned method C failed the heavies, fast method A ~0.03 s passes); Rust/Scala/Clojure/
Java/Kotlin/Dart/Go rows are 100% with method C wrappers, so their limits allow ~2 s spawned perl.
