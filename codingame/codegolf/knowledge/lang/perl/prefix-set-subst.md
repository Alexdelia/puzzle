# Distinct prefixes via `s///ge` and `~~%h`

Verified locally (telephone-numbers perl 37, CG row #11, 15/15 + CG confirmed).

- all prefixes of a line: `s/./$h{$`.$&}++/ge` (`$`` = text before the char, `$&` = the char); newline is not matched by `.`, so no empty key and no `\n` handling
- `for<>` aliases the lines, they are modifiable: `<>;s/./$h{$`.$&}++/ge for<>;print~~%h` (skips header with `<>;`)
- `~~%h` = number of keys in perl >= 5.26 (hash in scalar context), 5 bytes under `~~keys%h`
- `-n` with `}{print~~%h` and `$.>1&&` guard measured longer (46) than the no-shebang form (37): a `#!perl -n` line costs 10
- running the referee by hand: `CG_PERL` is the package root (`dirname dirname readlink -f perl`), not the binary
