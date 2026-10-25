# GNU sort -V on signed integers

`sort -V` (version sort, coreutils) on one signed decimal integer per line orders:
**all non-negative numbers ascending by value first, then negative numbers ascending by
magnitude**; on equal magnitude the positive line sorts first (tie goes positive).

Verified locally 2026-10-25 (coreutils 9.0, LC_ALL=en_US.UTF-8, locale archive):
- `printf '%s\n' -8 32 3 42 5 -8 -8 55 -2 12 | sort -V` -> 3 5 12 32 42 55 -2 -8 -8 -8
- `printf '%s\n' 7 5 9 -1 4 | sort -V` -> 4 5 7 9 -1
- `printf '%s\n' -1 1 | sort -V` -> 1 -1   (equal magnitude: positive first)
- `sort -k1.2V` compares from byte 2 of the line, i.e. the magnitude when byte 1 is a sign:
  `printf '%s\n' +4 -1 +12 -174 +7 +5 +9 | sort -k1.2V` -> -1 +4 +5 +7 +9 +12 -174
- `sort -V` through `xargs -n1`/`fmt -1`/`tr ' ' '\n'` is identical (tokens become lines).
- coreutils NEWS shows no sort change between 8.32 (CG bash box) and 9.0; not yet verified
  on CG itself (local tools are newer, see env.md).

Use: "minimum |t|, positive wins tie" = first line of `... | sort -V` **unless** a negative
has strictly smaller magnitude than every positive (then sort -V puts the smallest positive
first). GNU `-k1.2V` on `%+d`-formatted numbers does give the true answer (magnitude key,
sign at byte 1); `%+d` needs a formatter, so this costs ~50 B in bash on temperature.

`sort -Vr` is the exact reverse (largest positive first, then negatives by decreasing
magnitude); `sort -k1.2n` on raw tokens treats single-digit positives as key 0 (empty),
which is why `sort -k1.2n|head -1` is wrong for "closest to zero".
