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

Rule: lines compared exactly after dropping trailing empty lines. Free: final newline, any count of trailing empty lines.
