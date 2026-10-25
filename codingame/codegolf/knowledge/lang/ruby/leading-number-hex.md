# `$_.hex`: 6 B leading-number accessor (and its base-16 offset)

With `-0` (slurp) `$_` is the whole stdin, so `$_.hex` parses the leading number of line 1.
It is 1 B shorter than `$_.to_i` (6 vs 7 B) and 4 B shorter than `$F[0].to_i`, but reads
**base 16**: `"10\n..."` -> 16, `"12\n..."` -> 18, `"1a\n..."` -> 26. Safe when the value
is 0..9 or when the 10 -> 16 mapping is exactly what you need (e.g. an 8-vs-9 splitter).

- temperature ruby 46, CG 100% (task cell.temperature.ruby.1792878202232865737, 2026-10-25):
  `#!ruby -0an` + `l=eval$F[-1];p l-l[2]*5-$_.hex/9*9`; `$_.hex/9*9` is 9 on the only
  N=10 validator, 0 on every other (N <= 6), 10 B vs 11 B for `$F.size/8*9`.
- `$_.oct/8*9` and `$_.hex/16*9` also work on the same data; both are 11 B, no gain.
- same family: `$_.sum` (6 B, byte checksum), `$_.size` (7 B, total length incl newlines),
  `$.` (2 B, record number; 2 under `-0an`/`-0ap`, 2 then 3 under `-an`/`-ap`).
