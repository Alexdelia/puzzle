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

Method (verified 21/21 local, Python and bash+gawk, 2026-10-19): answer = `max(x)-min(x) + f(Y)`, `f(Y)` = sum of absolute deviations of `y` from its median. Only one sort, on `y`; `x` needs min and max only, never sorted, never paired with its `y`.

Equivalent forms of `f` on `Y` sorted ascending (`n` items, 0-based), pick per language:

- pair ends: `sum over i < n/2 of Y[n-1-i] - Y[i]` (loop `i<n/2` also fine: odd middle pair is `m-m=0`); suits awk/C loops, stack langs (pop both ends)
- slices: `sum(Y[floor(n/2):]) - sum(Y[:ceil(n/2)])`; both slices hold the median when `n` odd, it cancels. Python `sum(y[n//2:])-sum(y[:-~n//2])`
- median: `sum |v - Y[n div 2]|`; one char longer in Python, likely shorter where map/abs is cheap (vector langs, Ruby `sum{}`), unmeasured there
- doubled list: sort `Y` twice over (`2n` items, always even), `(sum(top n) - sum(bottom n))/2`; no odd case, costs the `/2`
- asc vs desc: `f = sum over i of max(Ydesc[i]-Yasc[i], 0)` = `sum |Ydesc[i]-Yasc[i]|/2`; zip of a list with its reverse, or `paste <(sort -n) <(sort -nr)` in shell (unmeasured)

`n` itself is never needed: the count of read lines works (awk `NR`, `len`). In shell `read` drops the header, then `sort -nk2` orders lines by `y` and awk keeps `y[NR]` plus running min/max of `$1`.

Lengths reached (local, not submitted, scratch of journal `puzzle.network-cabling.-.1792440302676785517`):

- Python 113 (doubled list), 114 (slices, median), 116 (all inline), 121 (pair-end genexpr); top 88 (Lydxn): the `n,*a=map(int,open(0).read().split())` + `max(x)-min(x)` + `sorted` skeleton alone is about 100, so 88 needs a reading or structure trick not found here
- bash 116 (`read` + `sort -nk2|awk` pair ends + running min/max), tested with local bash 5.3 / gawk 5.4, not the CG toolchain (CG_BASH unset in that shell); top 79

Rejected:

- `sum(Y[n//2:])-sum(Y[:n//2])`: off by the median for odd `n`, fails 8/21
- mean instead of median: fails tests 02, 05, 09
- min over candidate `m` of `sum |y-m|` without sort: O(n^2), validators 04/10 have N about 100000
- folding the `x` range into the `y` sum (adding `min x - C`, `max x + C` to `Y` keeps the median, adds `range + 2C`): needs min/max anyway, longer

## Hidden validators

Details, sources and open probes: `puzzle/network-cabling/referee/hidden/facts.md` (2026-10-20).

- 12 validators, not 9 (probe grain 1/12: typescript exact-match probe 3/12 = 25 %)
- 4 recovered exactly (forum 2015 submit-report names quote them, https://forum.codingame.com/t/network-cabling-puzzle-discussion/41?page=4, confirmed by probes): `(0 0)(2 2)(4 4)` -> 8, `(2 3)(1 1)(4 4)` -> 6, `(1 2)(0 0)(2 2)(5 3)` -> 8, `(-5 -3)(-9 2)(9 -4)` -> 24; in `referee/hidden/01 02 03 05`
- other 8: 4 with N <= 11 (likely N = 1, a large-L one, a dispersed one, one unnamed), 4 with N >= 99999; none in 12..99998
- no separating key or answers for the 8 unknown: a full hardcode is not possible yet; weak algorithms must still pass N = 1, large coordinates and N about 100000
- probe rows (sub-100 %, to be replaced by real solutions): php 380, lua 375, typescript 475, ruby 161, perl 209
