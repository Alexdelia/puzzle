# Assignment inside an index, integer indicator, bounded plan dump (Ruby)

Verified CG: power-of-thor ruby 61 B, 100% (2026-10-24).

- assign a variable while indexing: `h="WEN"[i=gets.sum%3]` (20 B) replaces `i=gets.sum%3` + `h="WEN"[i]` (two statements, 23 B incl newline); the value of `i=...` is the assigned integer, usable in the same line.
- `0**i` is the cheapest integer indicator `[i==0]` for i in 0..2 (values 1,0,0). It matters because `String#*` needs an Integer: `h*!i` and `h*(i<1)` raise TypeError. `h*0**i` = 6 B; `h*(1-i)` = 7 B also works for i<=2 (`String#*-1 == ""`); `h*(i<1?1:0)` = 11 B.
- whole plan dump without `$>.sync`: `puts ARRAY*999` (48 elements -> ~96 KB) fills the pipe buffer, so the write blocks (process stays alive) and data flushes; verified on CG. `loop{puts ARRAY}` costs 2 more bytes. Plain `puts ARRAY` (150 B) relies on exit-flush and closed-stdin ordering, do not use for interactive.
- `Array#*` negative raises, `String#*` negative returns "": for a possibly negative run count use strings (`$/` costs 2 B per use) or clamp; for a known non-negative table count use arrays.
