# Distinct prefixes via `s///ge` and `~~%h`

Verified locally (telephone-numbers perl 37, CG row #11, 15/15 + CG confirmed).

- all prefixes of a line: `s/./$h{$`.$&}++/ge` (`$`` = text before the char, `$&` = the char); newline is not matched by `.`, so no empty key and no `\n` handling
- `for<>` aliases the lines, they are modifiable: `<>;s/./$h{$`.$&}++/ge for<>;print~~%h` (skips header with `<>;`)
- `~~%h` = number of keys in perl >= 5.26 (hash in scalar context), 5 bytes under `~~keys%h`
- `-n` with `}{print~~%h` and `$.>1&&` guard measured longer (46) than the no-shebang form (37): a `#!perl -n` line costs 10
- running the referee by hand: `CG_PERL` is the package root (`dirname dirname readlink -f perl`), not the binary
- `/s` variant (2026-10-22, CG 100% at 31, telephone-numbers): `s/./\$h{$`}/sge` - `.` with `/s` also matches the trailing `\n`, so `$`` alone runs over prefixes of length 0..len (empty one included: subtract 1). 2 bytes under `$`.$&` keyed form
- `\$h{K}` (take a ref) autovivifies key K: 1 byte under `$h{K}++` when only the key set matters, valid in `/e` replacements
- `sgefor` without space dies: "Unknown regexp modifier /f"; keep the space before `for` after a modifier list
- empty-pattern variant (2026-10-25, CG 100% at bash 33, telephone-numbers submission 41564559): `s//\$h{$`}/ge` under `-l` (chomped `$_`). An empty pattern with `/g` matches zero-width at every position `0..len`; `$`` is the prefix, so the keys are all prefixes plus `''` and the full line. 13 B vs 14 for the `/s` dot form (1 under). Needs `-l` or an explicit `chop`: without chomp the position after `\n` adds the full line *with* `\n` as a key (one per distinct line, not a constant). Native perl (no flags) therefore keeps the `/s` form; this form wins in `perl -lnE'...'` wrappers
