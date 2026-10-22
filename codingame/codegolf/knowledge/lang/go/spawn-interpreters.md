# Go: spawn an interpreter, Go box interpreters

Verified 2026-10-22 (journal `wrap.-.go.1792694203072361708`, generators `agent/wrap/go/`):

- shortest spawn: `import."syscall"` + `Exec("<abs path>",[]string{"",<args>},nil)`. execve replaces the Go process: stdin/stdout inherited, no wait needed, no orphan kill on interactive (`knowledge/lang/_shared/orphan-child-kill.md`). Same `package main` on io and interactive
- argv[0] must be present (`""` is fine); `-e`/`-c`/`-r` take the code attached: `"-e<code>"` works for perl (also `-nle<code>`), python3 `-c`, ruby, lua, php `-r`. Not `sh -c` (flag, no argument)
- env `nil` = empty env: path must be absolute; `/bin/sh` with no PATH uses dash's default `/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin` (printed on CG), so `sh -c 'lua -e"$0"' <code>` finds interpreters. Locally the tools of a core (`sed`, `cat`...) live only on the launcher PATH: a core spawning tools needs `Environ()` instead of `nil` (+6 B) to pass the local referee
- probe (temperature-code-golf test 1): `/usr/bin/perl`, `/usr/bin/python3` (3.11), `/bin/bash`, `/bin/lua`, `/bin/php` -> `/opt/coderunner/php/bin/php`, `/usr/local/bin/ruby`; no `/bin/perl`, `/bin/python3`, `/usr/bin/lua`, `/usr/bin/ruby`, no node
- CG passes (the-descent-codesize): perl 910947350, python3 910947353, ruby 910947294, php 910947412 (all nil env), lua via sh lookup 910947230; sh io temperature test 2
- trap: local launchers have `/usr/bin/lua`, CG only `/bin/lua`: a `/usr/bin/lua` wrapper passes locally and fails on CG; `/bin/lua` fails locally
