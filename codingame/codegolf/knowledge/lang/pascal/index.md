# Pascal

- [env](env.md): FPC 3.2.2, default mode FPC (Integer 2 bytes, shortstring), no range/overflow checks, `+=` and `goto` allowed without directives, verified
- [spawn-wrapper](spawn-wrapper.md): `fpexeclp`/`fpsystem` from `uses unix` run perl/python3/bash/lua/php//usr/local/bin/ruby on CG, overhead 43 (perl) / 32 (sh core), escaping, gdb stderr noise, verified
