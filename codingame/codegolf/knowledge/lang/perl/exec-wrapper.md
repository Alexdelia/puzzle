# exec wrapper (perl host runs another interpreter)

Verified 2026-10-22 (CG Perl box, test games below; local CG perl 5.34):

- `exec LIST` = execvp, no shell, replaces the perl process: child keeps stdin/stdout, nothing to wait for, so interactive puzzles work (no orphan kill, `knowledge/lang/_shared/orphan-child-kill.md`)
- `-c` / `-e` as bare words are filetest operators (`exec bash,-c,...` runs `bash ''`); `-c=>` / `-e=>` / `-r=>` autoquote to the string, 1 under `"-c",`
- forms, cost over the core: `exec'core'` +6 (`/bin/sh` = dash when the core has a shell metachar), `exec lua,-e=>'core'` +15, `exec bash,-c=>'core'` +16, `exec python3,-c=>'core'` +19, `exec php,-r=>'?>core'` +18 (`?>` leaves PHP mode so the core keeps its `<?` tag)
- `'...'` escaping: `\'` for quote, double only backslash runs of 2+ or before a quote/end of string; `q~...~` when the core has `'` but no `~`; heredoc `exec<<''⏎core⏎` (+10 total for `exec`) needs no escaping, core must have no empty line, and the file needs the newline after the body
- `exec'core'` parses as exec + string (keyword wins over the old `'` package separator); `print STDOUT'x'` does not
- CG Perl box has on PATH: bash (game 910933623), lua (910933626), python3 (the-fall-episode-2 perl accepted), php (temperature test 1 io success). ruby: unverified, `exec ruby,...` timed out silently in game 910933627 (exec failure likely)
- a bash core that loops with `. $0` fails under `bash -c` ($0 = `bash`)
