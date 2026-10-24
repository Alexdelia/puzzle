# Packed 4-bit answer table + `vec`

Verified on CG 2026-10-24 (genome-sequencing perl 52 B, probe 100%, rank 2).

- `vec"<bytes>",KEY,4` returns the KEY-th nibble (low nibble of byte 0 first, then high nibble, ...), so a per-input answer table of 4-bit values is a raw byte string, not a number: no shift/mask needed
- values up to 15 fit; a 0..15 key chooses the slot. Table bytes = `ans[2i] | ans[2i+1]<<4`
- source is a double-quoted string; write unprintable bytes escaped: NUL `\0` (2), 0x01-0x1a `\cX` (3), high/DEL `\xNN` (4), printable 1 except `"` `$` `@` `\` (2 as `\"` `\$` `\@` `\\`)
- numeric alternative `$_=0x<hex>>>4*KEY&15` handles values 0..15 too but costs the `&15` mask (+4 vs vec)
- find the key by brute force: cheap invariants `y///c` (L), `y/A//`..`y/T//`, `$_` (=first number N), `y/A-Z//` (letters), `y/\n//`; test consistency of key->answer over the known hidden inputs
- genome-sequencing: `5*y/A//%8+y///c-9` (range 0..13) with 6-byte string `1\cB\0Ug5\x9B` gives all 14 hidden answers in 52 B
