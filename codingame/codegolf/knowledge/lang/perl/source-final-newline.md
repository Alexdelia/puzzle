# CG runs the source with a final newline

Verified 2026-10-24 on CG (chuck-norris perl 66, submissions 41558939 and 41558953, rejected twice, IDE test 1: stdout `0 0 00 0000 0 00`, "Found: Nothing").

CG appends a newline to the submitted code even when the file has none (`submit.sh` sends the file bytes exactly). Code that relies on the `-p`/`-n` wrapper text following the last character directly breaks:

- `#!perl -p` ending in `s; ;` without final newline: locally the wrapper's `;` closes the substitution, replacement empty, leading space removed. On CG the replacement is the newline, so output starts with an empty line and every test fails.
- same for any construct left open at end of file (`s;x;`, `y;a;`, unterminated quote-like ending at EOF)

Local referee matches CG since 2026-10-24: `agent/tool/lang/perl/build.sh` appends the newline when missing. Length is unaffected (trailing newline free, see `_shared/length.md`).
