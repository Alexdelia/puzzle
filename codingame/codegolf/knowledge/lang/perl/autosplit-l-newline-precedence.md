# `-a` implies `-n`; `-l` and `x` precedence facts

Verified locally with the CG perl 5.34 (`$CG_PERL/bin/perl`), 2026-10-24,
while golfing power-of-thor.

- `#!perl -a` alone is enough: autosplit implies the `-n` line loop and fills
  `@F` for every input line. It does not auto-print. So `-a` (10 B including
  the shebang newline) replaces `-an` (11 B) or `-ap` (11 B) whenever you
  print yourself, and `-ap`/`-alp` only win when the implicit `print $_` of
  `-p` is used. Measured: `#!perl -a` + `print"[$_]{@F}\n"` printed one line
  per input line.
- `-l` sets `$/` (chomp) and `$\` (one newline appended per `print` call).
  It never inserts a separator *between* list elements, so a multi-line
  string built by `x` still needs a newline inside the repeated unit
  (`"S$h\n"x$n`, literal newline = 1 B) or a `print for LIST` loop
  (`print for(S.$h)x$n`, which aliases `$_` per element and lets `-l` add
  each newline).
- `x` binds tighter than `.` and binary `-` (it is a multiplicative
  operator), so a repetition count containing a subtraction needs parens:
  `"S$h\n"x($F[1]-$F[3])`. A stored or single-token count needs none:
  `"S$h\n"x$t`. Parenthesized string operands are also unnecessary:
  `"S$h\n"x...` is 2 B under `("S$h\n")x...`.
- `-M` on the `#!` line is fatal (`Too late for "-M..."`), including
  `-Mfeature=say`, so `say` is not reachable from a shebang.
