# Ruby -> bash core via `exec"bash -c 'CORE'"`

Verified on CG 2026-10-25 (shadows-of-the-knight-episode-1 ruby 97 = 16 B wrapper + 81 B core, 100 %, local 16/16 + hidden 13/13).

- shortest Ruby wrapper for a bash-only core: `exec"bash -c 'CORE'"` = 16 + core bytes (`exec"` 5, `bash -c ` 8, two shell quotes 2, closing Ruby quote 1; the core needs no escaping when it has no `"`/`#`).
- the space after `-c` is mandatory: `exec"bash -c'CORE'"` puts the shell word `-cCORE` on bash's argv -> `invalid option` (verified bash 5.1.16; dash 0.5.11 same). `bash -c CORE` without quotes word-splits the core.
- `exec"CORE"` (6+core) is 10 B shorter but runs `/bin/sh` = dash; only for a dash-compatible core.
- Ruby has no Perl `$|`: `$|` is a NameError (`$|' is not allowed as a global variable name`); flush is `$>.sync=1` (9 B). Do not try `$|=1` from Perl habits.
- wrapper lengths for the same core in other hosts (measured from `agent/wrap/<host>/bash.sh`): php 25+core (`<?system('bash -c \'CORE\'');`), python 33+core (`import os;os.system("bash -c 'CORE'")`, 35 with single-quoted shell quotes), luajit/lua 22+core, groovy 54+core, ocaml 23+core.
