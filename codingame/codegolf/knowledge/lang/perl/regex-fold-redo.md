# Fold lines in order with one regex + redo

Verified 2026-10-22 (genome-sequencing, CG 100%): `#!perl -0p` slurps the input, `s/PAT//m&&redo` repeats one substitution until it fails, with no `1while` (1 byte shorter). A pattern anchored on the first two lines folds the list left to right: each pass rewrites `line1<LF>line2` into one line.

- leftmost-match = maximum: `(.*)\K<LF>\1` deletes the newline plus the longest suffix(line1) = prefix(line2) overlap, since the earliest start in line 1 is the longest overlap; the empty overlap always matches at the end of line 1, so later lines are never touched first
- optional group + conditional fuses two alternatives: `(.*)(.+)?\K<LF>\2(?(3)$)` = merge when group 3 is unset, "line 2 occurs inside line 1" (line 2 must then end at `$`) when group 3 is set; `(?(N)..)` tests participation, so `(.*)` matching empty would always count as set, use `(.+)?`
- a header line glued by the first merge can stay: count with `$_=y/A-Z//` (letters only) instead of `length`, and skip it in `^`-anchored alternatives with `^\d?`
- literal newlines instead of `\n` in the pattern, see [literal-newline](literal-newline.md)
