# Memory limit

Probe 2026-10-21 on `roller-coaster-codesize` test 1, Python3 `a=b"\1"*(M<<20);print(7)` (pages touched), journal `referee.roller-coaster.-.1792589750622861841`:

- M=700 pass, M=740 pass (local peak RSS 768616 kB), M=770, 850, 1000 fail: no stdout, no stderr, `timeout` null (process killed, no MemoryError)
- limit about 768 MiB peak RSS per test (between 751 and 780 MiB)
- Haskell over the limit can show as `timeout: true` instead (roller-coaster: 1.27 GB peak, 3.1 s locally, CG reports timeout)
- local referee-core enforces no memory limit (2026-10-21): a solution peaking over ~750 MB passes locally and fails on CG. Check peak with `command time -f %M` before submitting anything that builds big lazy lists or arrays
