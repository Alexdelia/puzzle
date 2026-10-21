# io time limit

Probe 2026-10-19 on `chuck-norris-codesize` test 1, Python `time.sleep(s)` then solution:

- 5.95 s pass, 6 s timeout
- limit about 6000 ms per test, interpreter startup included
- referee uses 6000 ms for every io puzzle unless a puzzle note says otherwise
- per language, not global (verified 2026-10-21, super-computer-codesize): Groovy deadline ~1.9 s of JVM uptime, Python3 `sleep(3)` on time on the same test. See `lang/groovy/io-time-limit.md`. Other languages not probed: the 6000 ms is verified for Python only
