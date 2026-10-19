# Network Cabling

pretty_id `network-cabling-codesize`, leaderboard `network-cabling-codesize`, io.

## Rules (as referee)

- input: line `N` (`0 < N <= 100000`), then `N` lines `x y`; output: one integer, `max(x)-min(x) + sum |y - m|` with `m` a median of the `y`
- for even `N` any `m` between the two middle `y` (inclusive) gives the same sum; mean instead of median fails tests 02, 05, 09
- duplicate points and equal `x` count separately (statement note: no shared dedicated cable)
- answers pass 2^31 (tests 06, 08, 09); 64-bit is enough (max about 100000 * 2^31)
- io limit 6000 ms per test (shared rule, `lang/_shared/io-time-limit.md`, not re-probed here); Python sort of 100000 values is about 0.13 s locally

## Input quirks (captured 2026-10-19)

- single space between `x` and `y`, every test ends with a final newline, no CR
- test 06: N=8, coordinates near +-1e9 (statement range is +-2^30)
- test 07: N=100000 random `x,y` in [-63,63], unsorted. Local fixture holds the same point multiset (exact: histogram sha checked, same char length, same answer) in a different order: an order-dependent wrong solution could differ from CG
- test 08: N=99999, line `x 1-x` for `x` from -49999 to 49999 in order (exact, sha256 checked)
- test 09: N=11, `y` sorted ascending, values up to 2.7e8
- `capture-io.sh` cannot capture 07/08 (CG output cap about 4 kB); journal `referee.network-cabling.*` has the probe and histogram capture scripts

## Local validators

- 01 N=1, 02 N=2 same `x`, 03 corners +-2^30, 04 N=100000 random +-2^30, 05 N=100000 one point repeated, 06 all `x`=0, 07 negatives only, 08 same `x` with duplicates, 09 `y` strictly descending from 2^30, 10 N=99998 random [-63,63], 11 all `y`=0, 12 two points repeated 3 times at opposite corners

## Shortest known

- ours Python 232 (plain reference, 2026-10-19), rank 499
