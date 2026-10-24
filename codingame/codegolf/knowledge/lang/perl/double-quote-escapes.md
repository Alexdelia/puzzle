# Double-quoted regex strings drop `\H`, `\d`, `\w` backslashes

A regex embedded in a double-quoted string only keeps the escapes the string layer knows (`\n`, `\t`, `\\`, ...). Unknown ones lose the backslash:

```perl
$f="(?<=X\H{31})";   # becomes "(?<=XH{31})"  (warns "Unrecognized escape \H passed through")
$f="\d\w\n";         # becomes "dw" + newline
```

Consequences:
- Keep regex strings with `\H`, `\d`, `\w`, `\b`, `\s` in single quotes (`'...'`), or double every backslash (`\\H`, +1 B each).
- `\n` in a double-quoted regex string becomes a real newline, which the regex matches as a newline: 1 B instead of the 2 B `\n` inside a single-quoted string. Useful when a newline literal is needed in a pattern or a `y///` search list (`y/$chars\n/X/` with a raw newline).
- Verified on CG perl 5.34 and host perl 5.42 (`-w`), the-labyrinth 2026-10-24.
