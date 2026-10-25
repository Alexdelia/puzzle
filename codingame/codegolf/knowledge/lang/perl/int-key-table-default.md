# Integer-safe periodic hardcode key + `//1` default slot

For hidden-validator hardcodes keyed on a line length L (`length<>` = L+1):

- Perl `**` is too imprecise for big values (`pow-nv-precision.md`). Use
  chains of `* / %` with small constants instead. `n*a%b` depends only on
  `n mod 256` when `b | 256*a` (e.g. `*9%64`: `256*9 % 64 == 0`), so a
  validator whose true length is only known mod 256 (L = c + 256k, k unknown)
  still gets one fixed slot. Exact in 64-bit integers as long as
  `n*a` stays below 2**53.
- `(LIST)[K]//1` (or any fallback) turns an out-of-range index into the
  fallback value. Array subscripts truncate toward zero; valid negative
  indices -len..-1 wrap from the end, so "out of range" means K >= len or
  K <= -len-1. Put the most common/cheapest answer (here 1) there and the
  table drops the extra identical slot: the 7-slot table + `//1` costs 3 B
  for the default and saves 2 B per removed `1,` slot.
- `length<>` needs parens before `* % / + - << >>` but not before `& | ^`
  (named unary precedence, `named-unary-precedence.md`).

Verified locally on CG perl 5.34 and on CG by submit (the-resistance perl
65 B, 100 %, rank #2 tie #1, 2026-10-25).
