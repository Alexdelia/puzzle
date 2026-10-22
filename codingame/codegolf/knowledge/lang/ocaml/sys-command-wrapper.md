# Sys.command wrapper: 14 bytes overhead

Verified on CG 2026-10-22 (ascii-art OCaml 84 = 100%, rank 1, perl core): the CG OCaml box has `perl` on PATH and `Sys.command` runs it with inherited stdin/stdout.

- whole program: `Sys.command"perl -lnE'<core>'"`, a bare top-level expression (no `let _=`, no `;;`): non-unit top-level expression compiles without error
- overhead `Sys.command"` + `"` = 13 + 1, shell is `/bin/sh` (dash, see `_shared/system-wrapper.md`): perl switches go on the command line for free (`-lnE`, `say`)
- `\` in the core: OCaml keeps illegal escapes like `\p`, `\d`, `\s` verbatim with warning 14 (not an error); legal ones (`\n`, `\t`, `\\`, `\"`, `\'`, `\b`, `\r`, digits) are still interpreted. `"` in the core needs `\"`
- cell length = 14 + `perl -lnE''` (11) + perl core; compare with the perl cell core (switches excluded)
