# `bytes.split()`/`str.split()` with no argument drops empty fields

`b'a  b'.split()` == `[b'a', b'b']` (runs of ASCII whitespace collapse, empty fields are removed);
`b'a  b'.split(b' ')` == `[b'a', b'', b'b']` (explicit separator keeps them) but costs +4 B.

Consequences for keyed plan tables (`b'DATA'.split()[key]`):

- a 5-way key cannot be served by 4 plans plus an "empty slot": the empty chunk disappears and the
  indices shift. A 5-value selector (`int(input()[-1])%5`) therefore needs 5 non-empty chunks (+1 real
  plan or duplicate, not just one separator).
- with 4 chunks the selector must produce 4 distinct indices that all fit a 4-element list: exactly
  `[-4..-1]` or `[0..3]` (or any 4 consecutive values inside that range). `ord(input()[-1])%5` maps
  digits 0,6,8,4 to {1,2,3,4}: 4 consecutive values but index 4 is out of range, so it still needs a
  5th chunk; the 22 B `ord(input()[-1])%5&3` (or `-1`/`-2`/`-3`/`-4` variants, same length) is the
  floor found for 4 chunks.
- plan tables whose chunks contain whitespace bytes (tab/LF/VT/FF/CR/space) split inside the plan under
  `.split()`; a plan byte that is whitespace must be dropped/moved, or use an explicit separator (+4 B).

Verified the-fall-episode-3 python 2026-10-25 (the 118 B base-10 candidate was invalid: `12`=FF inside
the v2 chunk split it, and the "empty 5th chunk" of the 117 B candidate vanished).
