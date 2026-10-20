# Perl one-liner as the bash solution

- CG bash has perl 5.34 on PATH: `perl -pe'...'` costs 10 bytes of wrapper over the perl code, verified (dwarfs, CG accepted 2026-10-20)
- under `-p`, line 1 lands in `$_`, so `@e=<>` takes the rest: a header line `N` is skipped for free
- `-p` prints `$_` at the end: put the answer (or a counter) in `$_` instead of `print`. `grep`/`map`/`for` localize `$_` and restore it, so `$_=2;$_++while@e=grep{...}@e` counts in `$_` safely
- missing final newline in the output is free (`knowledge/lang/_shared/output-compare.md`)
- worth it when the perl core is at least ~30 bytes shorter than the best pure bash
- `perl -lnE'...'` costs 12: `-l` chomps each line (needed when the line feeds a regex quantifier or a negative index), `-E` gives `say`, 1 shorter than `print`; verified (ascii-art, CG accepted 2026-10-20)
- under `-n`, header lines by `$.`: `$l||=$_;$.<4?<header>:<row>` keeps line 1 in `$l` for 8 bytes
- perl tricks apply unchanged inside the wrapper: see `knowledge/lang/perl/index.md` (e.g. `clear-underscore-autosplit`, `integer-division`); `-Minteger` goes before `-ape`
