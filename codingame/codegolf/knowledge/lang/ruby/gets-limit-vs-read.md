# `gets(n)` is a line with a max length; `$<.read(n)` reads n bytes across lines

Verified Ruby 3.1.2 local 2026-10-24, feeding `100\n10 240000\n`:

- `gets(5)` -> `"100\n"` (the first line; `gets` stops at `\n` even when the limit is larger)
- `$<.read(7)` -> `"100\n10 "` (exactly 7 bytes, crosses the newline)

So a byte-prefix read of the raw stream costs `$<.read(n)` (10 chars for a 1-digit n), never
`gets(n)`. Use `gets(n)` only to cap a single long line. Trick from a wrong assumption: keys
like `gets(7).sum` read line 1 only and cannot see line 2.
