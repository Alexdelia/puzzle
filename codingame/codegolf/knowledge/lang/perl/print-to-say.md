# print -> say under -E: -2 bytes when printing $_

- A Perl one-liner invoked as `perl -e'...'` whose last statement is bare `print` (prints `$_`) can use `perl -E'...;say'` instead: `-e`->`-E` costs 0, `print`(5)->`say`(3) saves 2 B. `say` appends the newline; CG accepts one trailing newline (`knowledge/lang/_shared/output-compare.md`).
- Verified on CG 2026-10-24: roller-coaster ruby `exec"perl -E'...say'"` 120 -> 118, score 100, submission 41560086, rank 10. Local 16/16.
- Same -2 for every wrapper that embeds the core in an explicit `-e` invocation; local 16/16 on roller-coaster: bash 114->112, lua 126->124, ocaml 127->125, swift 137->135 (cells queued 2026-10-24).
- Not usable in a `main.pl` run as `perl main.pl`: there is no `-E`, and `-E` on the `#!` line is fatal (`knowledge/lang/perl/shebang-no-E.md`); `use feature'say'` costs far more than 2 B. So a perl cell keeps `print`; only wrappers that pass the code to `perl -E` gain.
- Check with `puzzle/<p>/referee/run.sh <file> --lang <wrapper-lang> --validators`; the embedded `say` runs under CG perl 5.34.
