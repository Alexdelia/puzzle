# `-M` cannot go on the `#!` line

Verified locally (perl 5.42, 2026-10-20): `#!perl -Minteger -ap` dies with `Too late for "-Minteger" option`. `-a -p -n -l` are fine on the shebang.

- a bash wrapper `perl -Minteger -ape'...'` does not port to `main.pl` as is
- `use integer;` in the body costs 12 vs `-Minteger ` 10, and exact division without it (`while$r=($r-$r%20)/20` instead of `while$r/=20`) costs 10: keep the exact-division form (mayan-calculation: 193 bytes in `main.pl`, 21/21)
