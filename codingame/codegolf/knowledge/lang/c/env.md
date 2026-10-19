# C runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- gcc 11.2.0, `__STDC_VERSION__` 201710, `__STRICT_ANSI__` unset (GNU dialect, so `typeof`, statement expressions, `?:` elvis, nested functions should work: inferred, not run on CG)
- no optimization (`__OPTIMIZE__` unset), no stack protector, no `_FORTIFY_SOURCE`
- K&R golf compiles and runs: implicit `int` globals and params (`p=2;main(c,i){...}`), implicit declarations of `printf`/`gets`, no `#include`
- `gets` links (glibc 2.36 keeps the symbol), safe for line input
- PIE on, ASLR off: `main` at `0x5555555551a9`, addresses do not fit in `int` (no `int p=s` pointer tricks), but are stable across runs
- stderr is shown in the test-game `output` but not judged
- local: `agent/tool/lang/c/build.sh` reproduces all of the above except glibc (2.33 local) and exact code offsets (`main` at `0x555555555189` locally)
