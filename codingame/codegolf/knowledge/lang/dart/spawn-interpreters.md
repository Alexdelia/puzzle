# Spawning an interpreter from Dart

Form (verified locally, dart 3.9.4, and on CG 2026-10-22 via `agent/wrap/dart/`):

`import'dart:io';main()=>Process.start('perl',['-e',r'<core>'],mode:ProcessStartMode.values[1]);`

- `ProcessStartMode.values[1]` is `inheritStdio`, 4 chars under `ProcessStartMode.inheritStdio`; dot shorthand `.inheritStdio` is not on by default in 3.9
- the VM stays alive until the child exits (exitCode port pending): no `await`/`exitCode` needed, same form works on interactive puzzles
- wrapper cost: 89 chars + interpreter name/flags + core
- literals: `r'..'`/`r".."` take the core verbatim if it has no such quote and no newline; `r'''..'''`/`r"""..."""` allow newlines but must not end in the quote and drop a newline right after the opening quotes; non-raw strings need `\\`, `\'`, `\$`, `\n`

Interpreters in the CG Dart box (probe 2026-10-22, temperature-code-golf test 1, `sh -c 'command -v ...'`):

- on PATH: `perl` 5.34, `python3` 3.11.5 (`python` too), `bash`, `lua` 5.4.4, `php` 7.3.9
- ruby 3.1.2 only at `/usr/local/bin/ruby`: `'ruby'` = `not found`
- no `node`/`nodejs`
- local launcher inherits host PATH: plain `ruby` passes locally and fails on CG
