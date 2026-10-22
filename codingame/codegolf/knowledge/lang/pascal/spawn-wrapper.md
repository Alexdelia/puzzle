# Spawning an interpreter from Pascal

Verified on CG 2026-10-22 (temperature-code-golf test 1 success for perl/python3/bash/lua/php/ruby; the-descent test 1 game 910946412 win, shadows-of-the-knight-episode-1 test 2 game 910946413 win).

- `uses unix;begin fpexeclp('perl',['-eCODE'])end.`: exec replaces the process (PATH lookup, stdin/stdout inherited), so interactive needs no wait. Overhead 43 for perl, interpreter flag glued to the code (`-eCODE`, `-cCODE`, `-rCODE`, `-nleCODE`) saves 3 over a separate `'-e','CODE'`.
- `uses unix;begin fpsystem('CMD')end.`: `/bin/sh` = dash, waits for the child. Overhead 32 for a sh-compatible core; bash-only core: `fpsystem('bash -c ''CODE''')` (`'` in CODE costs `''\''''`).
- Pascal literal escaping only: `'` -> `''`, newline -> `'#10'` (juxtaposed char constant). Literals longer than 255 work in default mode (3000 tested, also as open array element).
- Do not compile in FPC 3.2.2: `shell(...)`, `popen` with one arg, `fpsystem` from `uses linux`. `dos.exec` splits args on spaces and truncates to shortstring.
- CG runs the Pascal binary under gdb: stderr shows `process 27 is executing new program: /usr/bin/perl` and `[Detaching after fork from child process 30]`. Not on stdout, not judged.
- Box (probe 2026-10-22, temperature test 1): perl 5.34, python3 3.11.5, python, bash, lua 5.4.4, php 7.3.9 on PATH; ruby 3.1.2 only `/usr/local/bin/ruby`; no node. `/bin/sh` -> dash.
- Generators: `agent/wrap/pascal/`.
