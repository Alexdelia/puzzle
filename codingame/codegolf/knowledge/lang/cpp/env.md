# C++ runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- g++ `__VERSION__` 11.2.0, `__cplusplus` 202002 (C++20), `__STRICT_ANSI__` unset (GNU dialect, gnu++20)
- no optimization (`__OPTIMIZE__` unset), no stack protector, no `_FORTIFY_SOURCE`, no `_GLIBCXX_ASSERTIONS`
- `#import<bits/stdc++.h>` compiles, typeless `main(){...}` compiles (no diagnostic shown, warnings appear off)
- `gets` does not exist in C++ (compile error, same message local and CG): use `getline`, `scanf`, `cin`
- PIE on, ASLR off: `main` at `0x555555556289` on CG, stable across runs
- stderr is shown in the test-game `output` but not judged; compile errors come back as `error.message` with `/tmp/Answer.cpp:line:col`
- local: `agent/tool/lang/cpp/build.sh` (`g++ -std=gnu++20 -w`, PIE, `setarch -R`) reproduces all of the above except glibc (2.33 local vs 2.36 CG), libstdc++ snapshot (`__GLIBCXX__` 20210728 local vs 20220414 CG, Debian 11.2.0 package with gcc-11 branch backports) and code offsets (`main` at `0x555555557269` locally)
