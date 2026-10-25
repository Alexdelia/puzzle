# Ruby `String#lines` keeps the newline

Verified Ruby 3.1.2 2026-10-25 (dwarfs golf, hidden inputs): `S.lines` returns each
line **with** its `\n`. Consequences for index/last-character idioms:

- `S.lines.max[-1]` is `"\n"` whenever the max line is not the final unterminated one;
  with a final newline it is always `"\n"` (useless)
- the last digit of the lexicographically largest line is `S.lines.max[-2]`
  (`[-2].hex` = that digit; `"2 3\n"[-2]` -> `"3"`)
- `S.split(?\n)` gives the same lines **without** `\n` (an extra `?` arg, no trailing
  empty field is kept by default), so use `split(?\n)` when `[-1]` should be the last
  character of the line
