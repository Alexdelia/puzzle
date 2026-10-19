# D runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- DMD 2.099.0 (`__VERSION__` 2099, vendor Digital Mars D): `import std;` works
- no `-O` (`D_Optimized` unset), no `-release` (asserts on, bounds checks on), not `-betterC`, not `-unittest`, no `debug`
- warnings are not errors: unreachable statement after `return` compiles
- PIE, ASLR on: addresses differ every run
- glibc 2.36
- stderr shown in test-game `output`, not judged
- local: `agent/tool/lang/d/build.sh` reproduces all of the above except glibc (2.44 local)
