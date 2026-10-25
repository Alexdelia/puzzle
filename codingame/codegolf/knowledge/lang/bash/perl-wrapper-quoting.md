# Quoting a Perl one-liner inside a bash solution

Prices for a bash cell `perl FLAGS'BODY'` (verified 2026-10-25 on power-of-thor, bash 5.1.16/CG 5.34):

- plain single quotes: `perl -nE'` + closing `'` = 10 B of wrapper; the body can contain `$`,
  backtick, `%`, parens, but **no `'`** (ANSI-C-style `$'` postmatch needs ANSI-C quoting).
- ANSI-C quotes: `perl -nE$'` + closing `'` = 11 B; `'` inside the body must be written `\'`
  (+1 per quote), everything else (including backticks and `$`) is literal.
- double quotes: `perl -nE"` + closing `"` = 10 B; verified literal inside the bash string:
  `$|`, `$.`, `$'`, `$;`, `$/` (bash only expands known specials/parameters). The backtick
  must be escaped `\`` (+1) and every Perl `"` must be `\"` (+2); a Perl `"S"x` therefore
  costs +2, which cancels the `$'` saving. Avoid inner `"` by `q(S)x(` (+1 over `"S"x(`) if
  the body has many quotes, otherwise ANSI-C is equal/better for the power-of-thor body.
- `perl -ne'...'` vs `perl -nE'...'`: `say` costs `-E` (+1) but saves the newline/`$/`
  argument, net -1 in the power-of-thor shape; `-l` is never cheaper (it needs `$_=` instead
  of `say`, same 3 B, and +1 flag).
- a phase/body character count must be done in *file* bytes: a single `'` in the phase is
  +1 byte under ANSI-C (power-of-thor: 9-byte `$'>-$.%34` is 10 file bytes).
