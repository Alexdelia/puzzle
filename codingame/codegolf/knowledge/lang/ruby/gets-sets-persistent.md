# Ruby: `Kernel#gets` assigns `$_` (one read, two stats)

Verified Ruby 3.1.2 locally (`printf 'abc\n' | ruby -e 'gets;p $_.size'` -> 4): plain `gets`
(no `-n`/`-p`) sets `$_` to the line. So one read feeds several line statistics without a second
`gets` (which would consume line 2): `gets;p $_.sum`, `$_ .size`, `$_[i]`, `$_=~/re/`.

- one-read probe combining a content bit and a length-keyed table:
  `gets;p($_.sum[B]>0?[table][$_.size*55%64/6]:-1)` (the-resistance sum-bit probes, 10 submitted
  on CG 2026-10-25; per-validator pass/fail = bit B of the line byte sum)
- cost vs `gets.size` key: `gets;` adds 5 and `$_.size` saves 2, so a table keyed this way is +3
  unless the code needs `$_` anyway
- `$_` is the *last* line read otherwise; reset by every `gets`/`readline`, not by `read`

- Verified Ruby 3.1.2 2026-10-25: `$<.readline`, `$<.gets` and bare `readline` also assign `$_`
  (like `gets`); `$<.read` and `$<.read(n)` do **not**. Consequence measured on super-computer:
  `T[K]` evaluates T before K, so a line-2 `gets` inside a table literal clobbers `$_` before the
  index can reuse line 1 (`p [gets&&1,9,3,7991+83717/gets.hex][$_.hex%7/2]` prints J1-based
  answers, wrong on all 8 validators). If the index must reuse line 1, the line-2 read has to use
  `$<.read...` (leaves `$_` alone) and costs more than the 2 B saved.
