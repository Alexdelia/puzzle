# Base-64 arithmetic literals and a one-read two-field slurp

Verified on CG bash 5.1.16 (`$CG_BASH`), CG 100% on network-cabling 2026-10-25 (probe n86).

- bash arithmetic (`$((...))`, and the 1-byte-shorter deprecated `$[...]`) parses `BASE#digits`
  for BASE 2..64; digits are `0-9a-zA-Z@_`. `$[64#x]` = 59, so a lookup table of 10-digit
  integers can be stored as space-separated base-64 words and decoded only after indexing:
  `t=(<w0> <w1> ...);echo $[64#${t[IDX]}]`. Decimal words are valid base-64 literals too, so
  small values can stay decimal. 44 decimal digits of 9 hidden answers become 27 chars
  (`5714781103` -> `5kEaeL`, `2500149996` -> `2l1kbI`, `2067588810` -> `1Xfera`, `12788481` -> `MOc1`,
  `24` -> `o`); saves ~9 B over a decimal table even after adding `$[64#` + `]`.
- `$[` avoids `$((` `))` by 1 byte; bash 5.1 still accepts it.
- `read -d. n a b` with delimiter `.` absent from the input slurps the whole pipe (read returns 1 at
  EOF, harmless without `set -e`) and word-splits it: `n` = first word, `a` = second, `b` = the rest.
  It is 1 B cheaper than `read n;read a b` and needs no second line buffer. Example: input
  `3\n0 0\n2 2\n4 4\n` -> n=3 a=0 b=$'0\n2 2\n4 4'.
- negative indexed-array subscripts `${t[-2]}` resolve to `t[len-2]`, so a key expression may go
  negative for a few classes; place those answers near the table end instead of adding an offset.
- `$[64#...]` prints the exact integer (no exponent form), unlike mawk `print` for >= 2^31.
