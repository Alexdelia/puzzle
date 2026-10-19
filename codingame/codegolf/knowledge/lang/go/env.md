# Go env on CG

Verified 2026-10-19 with test games on chuck-norris:

- `runtime.Version()` = `go1.18.1`, linux/amd64, 64-bit pointers: generics and `any` available, no `min`/`max`/`clear` builtins (1.21)
- `runtime.GOMAXPROCS(0)` = 1 on CG; local launcher sets `GOMAXPROCS=1` to match
- single-file `package main`, stdlib only; stderr shows in test-games `output` but is not judged (tests succeed)
- local launcher startup ~5 ms (compile time excluded)
