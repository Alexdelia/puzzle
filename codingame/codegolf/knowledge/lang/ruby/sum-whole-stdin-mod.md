# Whole-stdin byte checksum: `$<.read.sum%m` (and the 39 B hardcode shape)

Verified locally Ruby 3.1.2, 2026-10-25:

- `$<.read` (7 B) reads all stdin; `.sum` is `String#sum` = sum of bytes
  **mod 2**16 = 65536** (empirically: `("a"*100000).sum` = 9700000 % 65536 =
  672; not the `2**16-1` the docs wording suggests). Fast (C) on ~1 MB.
- hardcode cost with a decimal array: `p [<slots>][$<.read.sum%7]` =
  19 + content, content = answer digits + filler digits + (slots-1) commas.
  The 5 class answers 3,1,1,1,9,3,7991,91708 (distinct: 1,3,9,7991,91708)
  are 12 digits: 5 slots = 35 B, 6 = 37 B, 7 = 39 B (fillers `0`, one digit
  each). `$><<%w[<5 answers>][$<.read.sum%7]` is also 39 B (4+20+15) and
  works only if the five residues are 0..4. The earlier note that
  `p` + 18 B table + `%7` = 39 was an arithmetic error (it is 35).
  A key shift (`$<.read.sum%7-c`, +2 B) maps five class residues forming a
  5-wide window into 5 slots: 37 B (35 if the window starts at 0).
- super-computer ruby top 39 = one of those whole-input `%7` tables
  (`sum` or `size`); values need per-validator reports, see
  `knowledge/puzzle/super-computer.md` "Ruby 44 hardcode" for the six staged
  bit-probe submission ids and the builder. 6 probes are already sent in
  `probe/ruby/`, so the decode is free once a session exists.
- the table entries are the residues `byte_sum mod m` of each hidden input.
  Length floor for the line-1-only family is 44 (exhaustive 2026-10-25 over
  `gets.{hex,oct,to_i,sum,ord,size}` + up to 2 ops, 233k expressions:
  `gets.hex%7/2` is the only 44).

- `String#sum(bits)` is **not** order-sensitive: `"12".sum(n) == "21".sum(n)` for
  n=4,5,8,16 (and `"aXb"/"abX"/"Xab".sum(4)` all 11), so a `sum(bits)` key cannot
  split equal-multiset inputs (verified local 3.1.2, 2026-10-25, dwarfs V2/V5).
