# `sum` checksum as a bash key: 5-digit padding, leading zero = octal

Verified 2026-10-25, bash 5.1.16 + GNU coreutils `sum` (CG bash box, test game testcase 9 on
network-cabling-codesize, code `sum|od -c`): the first `sum` field is zero-padded to 5 digits
(`08777     1`), and `$[c%9]` with `c` read from that field parses it as an **octal** constant
because of the leading zero.

- `read c _<<<`sum`` is 16 B (one shorter than the `$(sum)` form); `c` is the checksum string
  including leading zeros, `_` the blocks count.
- `$[c%M]` uses octal when `c` starts with 0: `01466` -> 822 -> `%9` = 3. If the 5-digit string
  starts with 0 and contains 8 or 9 the arithmetic **errors** (`value too great for base`), so a
  checksum-keyed table must be verified per input.
- A checksum hardcode in bash: `read c _<<<`sum`` + `t=(<M base64 words>)` +
  `echo $[64#${t[c%M]}]`; cost 18+S+M+len(index) + 2 (S = word chars, M = slots, index `c%9` = 3).
  For a 9-answer puzzle that is 16+39+20+2 = 77 B.
- `sum` (BSD) matches GNU coreutils across 8.32/9.0/9.11 (same values on the same bytes).
