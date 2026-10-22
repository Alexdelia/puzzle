# OCaml system() wrapper

Verified 2026-10-22 (mayan-calculation, CG submit 100%, 205 B rank 1): the whole program can be one bare toplevel expression `Sys.command"perl -ape'<core>'"`. No `let _=` is needed: a non-unit toplevel expression compiles (4.14.2). The CG OCaml box has `perl` on PATH, and the child inherits stdin and stdout.

- wrapper cost = `Sys.command""` = 13 B + shell command
- the shell is `/bin/sh` (dash); see `../_shared/system-wrapper.md`
- inside an OCaml string, `"` and `\` need escapes: rewrite the core without them (perl `join""` -> `join$,`)
- core with `"`, `\` or newlines: quoted string `Sys.command{|perl -0pe'<core>'|}`, nothing to escape, +2 B over `"..."` (verified blunder-episode-1, CG 292 100% rank 1, 2026-10-22)
- `'` in the core (shell single quotes): `'\''`, +3 B each (perl `$'` -> `$'\''`)
- `\` followed by a non-escape char (`\p`, `\d`, `\s`, `\w`) stays verbatim in a `"..."` string, only warning 14 (illegal-backslash), not an error: perl `\pL` needs no escape (verified ascii-art, CG 84 100% rank 1, 2026-10-22). Legal escapes (`\n \t \b \r \\ \" \'`, digits, `\x`, `\o`, `\u`) are still interpreted
