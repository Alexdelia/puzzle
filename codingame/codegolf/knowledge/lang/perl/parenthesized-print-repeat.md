# Parenthesized `print` and `x`: cyclic schedules in one print

Verified on CG Perl 5.34 (`$CG_PERL/bin/perl`), 2026-10-24.

- `print(EXPR)xN` prints EXPR once: the `x` applies to `print`'s return value
  (void context; `-Mwarnings` says "Useless use of repeat (x) in void context").
  Costs a silent wrong answer when a plan is meant to be repeated.
- `print EXPRxN` / `print"str"xN` repeats, because `x` binds tighter than the
  list operator `print`. Chained `x` is left-associative: `"A"x3x4` = 12 copies.
- To repeat a concatenation, parenthesize it inside the print:
  `print(("A"xL1 ."B"xL2)x1e5)`. `x` binds tighter than `.`, so
  `"A"x3 ."B"x4 x1e5` would repeat only the last term.
- This is the cheapest way to print one periodic (cyclic) open-loop plan: it
  needs no loop, no `$|` (output >64 kB fills the pipe and blocks, see
  ../_shared/pipe-block-keepalive.md), and the tail `1e5` copies keep the
  process alive. Cost measured by file size (literal newline, space before the
  dot): k segments = `11+8k + sum(d)+sum(c)` (k=2: 27+..., k=3: 35+...)
  (d = printed rotate chars incl minus, c = digits of the count).
- `x9e9` materializes the whole repeated string and dies before turn 1
  ("solution exited"); use `x1e5` (450 kB, blocks on the 64 kB pipe).
- A numeric `x` count directly before the concatenation dot needs a space:
  `"s"x52."-15` lexes `52.` as a float ("String found where operator
  expected"); `"s"x52 ."-15"` works (+1 B per dot).
- Verified working form on the local mars-lander-episode-3 referee:
  `print(("0 4\n"x34 ."-1 2\n"x9)x1e5)` (33 B) lands validator A only
  (the 31-B `print(EXPR)x1e5` form without the outer paren prints once).
