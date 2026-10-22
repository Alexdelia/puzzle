# `$^T` as free +inf sentinel, shell sort feeding grep

Verified 2026-10-22 (super-computer, CG 100 %, 46 bytes, local perl 5.34).

- `$^T` = script start epoch (~1.8e9), writable: a free "infinity" start value for a running min/bound, no `$e=9e9;` init and no `$e&&` "unset" test. Costs 1 byte per use over `$e`, cheaper than any init after two uses
- `` `sort -nr` `` in list context = stdin lines sorted numerically descending by leading number, 10 bytes vs `sort{$b-$a}<>` 13. Perl must not read stdin before it (`<>;` buffers a whole block, the child sees the rest)
- the header line (`N`, no space) rides along in the sorted list: make the match part of the test, `/ /*X>=...` is 0 for it, so it drops without `<>;`
- `$'and` parses as package var `$'and` (old `'` separator): use `A?B:0` instead of `A and B` (same length) after `$'`
- literal control char `$<0x14>` for `$^T` is a syntax error in 5.34 ("Unrecognized character \x14")
