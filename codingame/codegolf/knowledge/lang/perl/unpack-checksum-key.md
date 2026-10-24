# `unpack` checksum as a hardcode key

Verified locally with CG perl 5.34 (2026-10-24).

- `unpack"%C*"` and `unpack"%16C*"` = plain byte sum of `$_` (no second arg uses `$_`; 9 chars), `unpack"%32C*"` = Adler-32, `unpack"%16S*"` = little-endian word sum, `%32L*`/`%16L*` dword sums
- useful as a hardcode key when an input statistic must separate many validators: genome-sequencing hidden byte sum `%23` and adler `%24` both classify all 14 validators consistently
- TRAP: `%` binds tighter than the list operator, so `unpack"%C*"%23` parses as `unpack("%C*" % 23)` and dies `Invalid type`; parens `(unpack"%C*")%23` are mandatory (same trap as named unary `length+1`)
- `vec"T",(unpack"%C*")%m,4` = nibble answer table keyed by checksum; costs 22+escaped_table+len(K), so only pays when the table is small (<=~12 slots) or the language has a shorter sum
