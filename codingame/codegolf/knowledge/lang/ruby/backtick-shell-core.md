# Backticks + `$><<`: shortest shell-pipeline wrapper

Verified on CG 2026-10-25 (super-computer ruby 48 B, 100 %, submission
`a11dbf83`): `$><<` + backtick command beats `exec"..."` by 6 B when the host
wrapper is only a shell pipeline.

- backticks run `/bin/sh` (dash on CG) and are double-quote-like: `#{}`, `#$x`,
  `#@x` interpolate, but a plain `$1`/`$2` stays literal, so an awk program
  needs no `\` before its fields. Shell expansion still happens in the shell.
- `$><<str` writes the captured stdout with no newline. A missing final newline
  is free (shared output-compare), so no `print`/`p` needed.
- shape: `$><<` + 1 backtick + shell command + 1 backtick = 6 B over the command
  (vs `exec` 6 + core with quote escaping, `system` 8).
- super-computer core: `sort -nr|awk '$1+$2>e&&e?0:$2&&e=$1'|wc -l` (42 B,
  dash-proof `$2&&` guard for the header) -> 48 B total, 15/15 local, CG 100 %.
  `p`+backticks is 49, `print` 49, `system` 50, `exec` 54 here.
