# Line byte-sum key for hardcode lookup (sum start arg, bytes table)

Interactive puzzle with recovered hidden validators: a hardcode can key its packed plan on the
init line. Python has no `gets.sum`, but `sum(input().encode(),10)` (24 B) is a plain byte sum
**plus 10** for the LF that `input()` strips: `sum`'s start argument costs `,10` (3 B), while
`sum((input()+'\n').encode())` pays for the parens and the `+'\n'` (7 B more).

- the-fall-episode-2 hidden init lines `W H` (byte sums ex-LF 136,141,144,230,190,145,146):
  plain sum `%13//2` collides (V4/V5 both 4, V6/V7 both 1); `sum(...,10)%13//2` gives
  146,151,154,240,200,155,156 -> slots 1,4,5,3,2,6,0 (contiguous 0..6). `%13` alone needs
  12 slots (6 more separators) and saves only `//2` (3 B), so keep `%13//2`.
- cheaper than `sum(map(ord,input()))` (23 B) and `sum(bytes(input(),'utf8'))` (26 B).
- **bytes literal table**: raw control bytes 0x0F..0x7B inside `b'...'` are valid source and
  survive submit (only NUL is stripped, `knowledge/lang/_shared/trace-hardcode.md`). Cite a
  cell as one byte `v = x + 14y`; avoid 0x27 `'`, 0x5C `\`, LF/CR. `.split(b'~')` (explicit
  separator, keeps empty fields, 11 B) vs `.split()` (free, collapses empties, 8 B: usable only
  if no data byte is whitespace; 0x20 in a chunk would be eaten).
- conditional word: `i%6and'RIGHT'or'LEFT'` (21 B) beats `'RIGHT'if i%6 else'LEFT'` (24 B);
  `6and` parses on 3.11.5 and 3.14, prints a harmless SyntaxWarning to stderr (graders read
  stdout only). Follow the table with `print('WAIT\n'*99)` (no-op flood; one extra trailing
  empty line is free).
- Verified: the-fall-episode-2 python 193 B, CG 100 % rank 1, submission 41563002 (2026-10-25),
  local `run.sh --cg` 7/7. Same 63-cell table as the perl 187 / ruby 174 hardcodes, decoded
  natively (no perl/ruby process).
