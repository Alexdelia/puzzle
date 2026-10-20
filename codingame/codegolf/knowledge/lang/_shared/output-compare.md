# Output compare (io puzzles)

Probes 2026-10-19 on `chuck-norris-codesize` test 1, Python:

| output | result |
|---|---|
| expected + `\n\n` | pass |
| expected, no final `\n` | pass |
| expected + trailing space | fail |
| expected + `\r\n` | fail |
| leading empty line | fail |
| expected + extra non-empty line | fail |

Probes 2026-10-20, Python, `print(expected + "\n"*n, end="")`:

| puzzle | n=2 | n=3 | n=10 |
|---|---|---|---|
| `blunder-episode-1-codesize` test 1 (4-line output) | pass | fail | fail |
| `chuck-norris-codesize` test 1 | - | fail | - |

Failure shows `"found":"End of line (\n)"`.

Rule: lines compared exactly. Free: missing final newline, final newline, ONE empty line after it (`\n\n`). Two or more trailing empty lines (`\n\n\n`) fail. Bash/perl blunder 271 scored 8.5% on CG for this (printed hundreds of trailing empty lines, passed locally).

Local `referee-core` enforces the same rule since 2026-10-20 (`compare.rs`: at most one trailing empty line free on actual): bash blunder 271 now fails locally (1/12), as on CG. All 84 committed io solutions rechecked: none relied on extra trailing lines.
