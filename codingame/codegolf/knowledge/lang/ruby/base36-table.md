# base36 integer table

Hardcode tables of large integers can store each value in base 36 and decode the selected one with
`String#to_i(36)`. One base-36 digit holds 0-9a-z, so a 10-digit decimal answer above 2^32 drops to
6-7 chars.

    p %w[15ciw30 7m3nl y6zlqi 8 0 8 0 6 2miflbj o][INDEX].to_i 36

- `p %w` needs the space (`p%w` parses as `p % w`); `%w[...]` is 2 B over a decimal integer array
  for the same slots, so the break-even is when the base36 text is >=2 B shorter.
- `.to_i 36` is 8 B (with parens 9) and runs on the selected element, after the slice.
- lowercase is fine; `to_i(36)` is case-insensitive.

Measured (2026-10-25, network-cabling): 10-slot answer table 93 B decimal -> **89 B** base36,
`puzzle/network-cabling` `agent/state/journal/cell.network-cabling.ruby.1792924378685756141.scratch/hard89.rb`
(hidden 01..05 5/5; keyed hardcode for V6-V9 unchanged). Base36 also wins on an 8-slot table:
decimal 52 B vs base36 40 B + 8 B decode = 48 B, still 4 B under.

Caveat: if the values are small (<5 decimal digits) base36 does not pay for the 8 B decode; count
`sum(len(base36)) + slots-1 spaces + 4 (%w[]) + 8` against `sum(len(decimal)) + slots-1 commas + 2`.
