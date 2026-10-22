# Spawning another interpreter from Python3

- `import os;os.system("perl -e'<core>'")` = core + 32 (perl flags fold in: `perl -anle'...'`). `os.system` runs `/bin/sh` = dash and waits for the child, so interactive puzzles work. Verified CG: mayan-calculation, dont-panic, the-labyrinth (committed python solutions)
- `import os;os.execlp("perl","p","-e",<core literal>)` = core + 39, no shell quoting: wins over `os.system` when the core has 3+ `'` (each costs `'\''` in the shell form). Verified CG 2026-10-22, the-descent-codesize test game 910932696 (interactive, perl core, success)
- `os.execlp` argv0 must be non-empty: `os.execlp("perl","","-e",...)` raises `ValueError: execv() arg 2 first element cannot be empty` (Python 3.11.5, checked locally with `$CG_PYTHON`). Argv0 `-...` would make bash a login shell: use a letter
- Python literal choice: `"..."` escapes `\ " newline`, `'...'` escapes `\ ' newline`, `r"..."` saves every doubled backslash when the text has no `"`/newline and does not end in `\`, `"""..."""` keeps newlines raw (1 char instead of `\n`) for 4 extra quote chars
- CG Python3 box (2026-10-22, temperature-code-golf testcase 1 from `os.system`): `lua` and `php` on PATH (`lua -e`, `php -r` succeed), `ruby` missing (`sh: 1: ruby: not found`)
- Generators: `agent/wrap/python/` emits all forms, `wrap.sh` keeps the shortest passing one
