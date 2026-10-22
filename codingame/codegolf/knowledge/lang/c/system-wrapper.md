# system() wrapper forms

Verified on CG 2026-10-22 (temperature-code-golf test 1 probe and generated candidates, the-descent-codesize game 910947232):

- C box PATH: perl 5.34, python3 3.11.5, python, `/bin/lua` 5.4.4, `/bin/php` 7.3.9, bash; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node/nodejs
- `main(){system("python3 -c'...'");}`, `lua -e'...'`, `php -r'...'`, `/usr/local/bin/ruby -e'...'` all pass; `system` waits for the child, so interactive works as is
- `system` runs `/bin/sh` = dash: bash-only cores need `bash -c '...'` (space needed: `-c'x'` would parse as option cluster), other interpreters take the attached form (`-e'x'`, `-c'x'`, `-r'x'`)
- quoting cost per char of the core: single-quoted shell `'` costs 5 (`'\\''` in C), `"` and `\` cost 2; double-quoted shell `"..."` costs `$` 3 (`\\$`), `"` 4, `\` 4, `` ` `` 3, `'` 1. Cores with many `'` and no `$` (python, lua) are shorter double-quoted; `agent/wrap/c/` emits both
- the test-game output starts with `[Detaching after vfork from child process N]` (gdb noise on stderr), not judged
- local trap: host gcc 16 rejects K&R golf (`implicit-int` is an error); compile with `$CG_C/bin/gcc -std=gnu17 -w` (gcc 11, as `agent/tool/lang/c/build.sh`)
