# exec wrapper forms

Ruby `exec` replaces the process (same pid, stdin/stdout kept): no wait needed on interactive puzzles, no orphan kill.

Interpreters on PATH in the CG Ruby box (CG test games 2026-10-22, all 100%): `perl` (910933624, dont-panic), `python3` (910933386, the-descent), `bash` (910933406, power-of-thor), `lua` (910933448, the-descent). `php`, `node` unverified.

Literal forms, overhead for a perl `-E` core (verified locally with a printf round trip, and on CG for the heredoc):

- shell: `exec"perl -E'..'"` (15): ruby `"` doubles every `\`, `"` -> `\"`, `#{` `#$` `#@` -> `\#`; core `'` -> `'\''` for sh
- argv, no shell: `exec"perl","-E",'..'` (18): core `'` -> `\'` (2 bytes, not 4), `\` doubled only before `\`, `'` or end
- `%q|..|` with a delimiter absent from the core: no quote escaping; `exec %q|` needs the space (`exec%q` parses as modulo), `exec"perl","-E",%q|` does not
- heredoc `exec"perl","-E",<<'E'` + newline + raw core + `\nE` (24): zero escaping. Shell form needs `exec <<'E'`: `exec<<'E'` parses as shift (`exec << 'E'`), ArgumentError
- a bash-only core needs `bash -c'..'` (space before the quote: `-c'x'` is a flag cluster); plain `exec"core"` runs dash
- a core that re-sources itself (`. $0`) fails in every form ($0 is `sh`/`bash`)

Generators in `agent/wrap/ruby/` emit all forms; `wrap.sh` keeps the shortest passing.
