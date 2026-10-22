# OCaml system() wrapper

Verified 2026-10-22 (mayan-calculation, CG submit 100%, 205 B rank 1): the whole program can be one bare toplevel expression `Sys.command"perl -ape'<core>'"`. No `let _=` is needed: a non-unit toplevel expression compiles (4.14.2). The CG OCaml box has `perl` on PATH, and the child inherits stdin and stdout.

- wrapper cost = `Sys.command""` = 13 B + shell command
- the shell is `/bin/sh` (dash); see `../_shared/system-wrapper.md`
- inside an OCaml string, `"` and `\` need escapes: rewrite the core without them (perl `join""` -> `join$,`)
- core with `"`, `\` or newlines: quoted string `Sys.command{|perl -0pe'<core>'|}`, nothing to escape, +2 B over `"..."` (verified blunder-episode-1, CG 292 100% rank 1, 2026-10-22)
- `'` in the core (shell single quotes): `'\''`, +3 B each (perl `$'` -> `$'\''`)
- `\` followed by a non-escape char (`\p`, `\d`, `\s`, `\w`) stays verbatim in a `"..."` string, only warning 14 (illegal-backslash), not an error: perl `\pL` needs no escape (verified ascii-art, CG 84 100% rank 1, 2026-10-22). Legal escapes (`\n \t \b \r \\ \" \'`, digits, `\x`, `\o`, `\u`) are still interpreted
- `python3` is on PATH in the CG OCaml box too, interactive io works: `Sys.command{|python3 -c '<core>'|}` = 28 B + core, `'` in the python core rewritten as `"` (verified the-fall-episode-2, CG 936 100% rank 4, 2026-10-22)

Quoted string literal (verified 2026-10-22, chuck-norris, CG accepted): `Sys.command{|...|}` takes the core verbatim, `"` and `\` included, for 2 B over `"..."`. Cheaper than escaping as soon as the core holds 2+ `"`/`\`; with exactly one `"` pair, a perl `q(...)` (+1) inside `"..."` ties it.
- trap: a core containing `|}` ends the `{|...|}` string early (compile error). Reorder the core (`$r.=X."|"}{` -> `$r.="|".X}{`) or use `{a|...|a}` (+2) (verified the-resistance, CG 179 100%, 2026-10-22)

OCaml box interpreters (verified 2026-10-22, `command -v` probe on telephone-numbers-codesize test 1, then one passing test per core): `python3` 3.11, `lua` 5.4.4 (`/bin/lua`), `php` 7.3.9 (`/bin/php`), perl 5.34, bash on PATH. `python` is Python 2.7.16: never `python -c` for a python3 core. `ruby` is off PATH, only `/usr/local/bin/ruby` (3.1, `_1` works): `Sys.command"/usr/local/bin/ruby -e'...'"`. No node/nodejs. `lua -e`, `php -r`, `/usr/local/bin/ruby -e` wrappers pass on CG. Generators: `agent/wrap/ocaml/`.
