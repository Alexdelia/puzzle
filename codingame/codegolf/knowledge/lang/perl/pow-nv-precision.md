# `**` is NV: do not port bignum pow keys

`X**Y` in perl 5.34 does not compute in integers: once the result passes
2**53 it comes back as a double and the low bits are gone.

- `66**8` = 360040606269696 is exact (< 2**53)
- `401**8` prints `6.68582463235588e+20`; exact is
  `668582463235588483201`, so the last digits are wrong.
- A ruby key of the form `gets.size**8/62%9` (n = line length, arbitrary
  hidden value) therefore **cannot** be copied into perl: `(length<>)**8/62%9`
  prints wrong slots for n = 401 and n = 9106 (verified: local hidden
  validator 09 gave 1 instead of 64, 10 gave 8 instead of 37265080320).
- Safe: any expression whose intermediate values stay below 2**53. Exact
  integer-safe periodic keys are cheap, e.g. `(length<>)*9%64` (256-periodic
  because `256*9 % 64 == 0`), then `/6%10`.
- `**` with a small result (n <= 2**17 and exponent 3, or n <= 64 and
  exponent 8) is exact; only the >2**53 range breaks.

Verified locally with CG perl 5.34 (the-resistance perl cell 2026-10-25, did
not submit the broken 62 B pow probe; the exact key scored CG 100 % at 65 B).
