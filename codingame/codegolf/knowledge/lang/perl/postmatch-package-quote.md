# `$'` before a word is a package name, `log` without arg reads the numeric prefix

Verified locally 2026-10-22 (blunder-episode-3, Perl 5.34 rules):

- `$x+=log$'for<>` does not compile as meant: `'` is the old package separator, so `$'for` is `$main::for`. Put a space (`log$' for<>`) or end the statement on another token. `$`` (prematch) has no such issue.
- `log` (and other numeric unary ops) with no argument use `$_`; on a line `"34 988\n"` Perl numifies the leading number, so `log` = `log 34`. Pair with `/ /` and `log$'` for the second column: `/ /,$a+=$s*log,$b+=$s*log$'`.
- `map{$_,"$_ $l"}LIST` is parsed as a block (not an anon hash) here and returns 2 items per element.
