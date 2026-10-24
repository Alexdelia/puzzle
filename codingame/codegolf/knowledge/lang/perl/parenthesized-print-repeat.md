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
  process alive. Cost `31 + d1+d2+d3 + c1+c2+c3` bytes for 3 segments
  (d = printed rotate chars, c = digits of the count).
- Verified working form on the local mars-lander-episode-3 referee:
  `print(("0 4\n"x34 ."-1 2\n"x9)x1e5)` (31 B) lands validator A only.
