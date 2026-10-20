# Exact integer division in Perl

Verified locally (perl 5.34, 2026-10-20).

- `/` of two IVs whose quotient is exact returns an IV, exact above 2^53: `($r-$r%20)/20` and `eval"$a/$b"` with exact division stay exact
- `-Minteger` on the command line (10 bytes with its space) makes every `/` integer, and `eval STRING` inherits it: cheaper than `($r-$r%20)/20` (+11) once a loop does `$r/=20`
- float `$r/20|0` or `int$r/20` is wrong above 2^53 (spacing of doubles)
