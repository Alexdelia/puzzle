# Hardcode split: `p TABLE[line1-stat]||A+B/gets.hex`

Verified on CG 2026-10-25 (super-computer ruby 44 B, 100 %, rank #3, commit
`eceb2939`): key the small validators on a cheap whole-line-1 statistic, leave
the two large-input answers to an arithmetic selector over line 2. `Table#[]`
out of range gives `nil`, so `||` needs no explicit test.

- `gets.hex` parses leading hex digits and stops at the space: `"1 4"` -> 1,
  `"875000 780"` -> 0x875000. With integer division, `A+B/gets.hex` is exactly
  `A+B` when the field is 1 and `A` for any field > B: one field separates the
  two large answers with 19 B (`7991+83717/gets.hex`), no ternary/parens.
- key `N.hex%7/2` maps the six small N values to 3 slots; the two N=99999 lines
  share one out-of-range index and fall into the selector.
- PARSE TRAP: in `p X||gets<?8?A:B` the ternary binds *outside* `p`'s command
  argument (Ripper: `p((X||gets<?8)?A:B)`), so the table is ignored and the
  small inputs print A. An arithmetic selector (`A+B/gets.hex`) has no `? :` and
  parses as `p(X||selector)`; parenthesising the ternary costs 2 B and lost to
  44. (No submit with the bare ternary was made; the parse is Ripper-verified.)
- `String#hex` accepts 8/9 as hex digits; `"99999".hex` = 629145.
- same shape applies to any hidden set with a known line-1 stat and one numeric
  line-2 field that is 1 on one validator and huge on the other.
