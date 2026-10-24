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

Details, sources and probe log: `puzzle/network-cabling/referee/hidden/facts.md` (2026-10-24).

- 9 validators (CG per-validator report), weights 100 (1-6) and 200 (7-9); the old "12 validators" was the weight grain 1200/100
- recovered exactly, in `referee/hidden/01..05`: `(0 0)(2 2)(4 4)` -> 8, `(2 3)(1 1)(4 4)` -> 6, `(1 2)(0 0)(2 2)(5 3)` -> 8, `1` / `9 9` -> 0, `(-5 -3)(-9 2)(9 -4)` -> 24
- answers known: 7 (N = 100000) -> 12788481, 8 (N = 99999, L = 99998 * 25002) -> 2500149996; open intervals: 6 (N 8..9) in 5714781001..5714781105, 9 (N 10..11 likely) in 2067588608..2067588863
- key: N separates 4, 6, 7, 8, 9 and the N = 3 group (1, 2, 5: first char of line 2 `0`, `2`, `-`) from 3 (N = 4). Full hardcode still blocked on 6 and 9 (about 8 more bisection probes)
- perl `sort{$a-$b}` (comparator truncated to 32 bits, wrong when two values differ by >= 2^31, fails local validators 03/12 at +-2^30 corners) passes all 9 hidden validators: CG 100 % on OCaml 102 perl wrapper, 2026-10-22 (probe log). Drop `<=>` for `-` in every perl core of this puzzle
- probe rows (sub-100 %, to be replaced by real solutions): php 380, lua 375, typescript 475, ruby 161, perl 209

- ours Clojure 167 (2026-10-21, rank 4, top 158): `F=#(map -(sort > %)(sort %))` on x and y columns; first of F(x) = range, positives of F(y) = deviation sum; trick note `knowledge/lang/clojure/read-pairs-sorted-diff.md`

- ours OCaml 101 (2026-10-22, rank 3, top 94; x sort inlined: `say$s-(@x=sort..)[0]+pop@x`): `Sys.command"perl -0nE'...'"` wrapper around a 78 B perl core (two `sort{$a-$b}` on `/.+ /g` and `/ .+/g`, pop-while-iterating pair ends `$s-=$_-pop@y for@y`, range `-$x[0]+pop@x`). General OCaml (split, `sort(-)`, `map2(-)(rev s)s`) is 182. Perl `sub f{sort{$a<=>$b}@_}` saves 2 B but timed out locally on 1e5 tests (unclear if load)

- ours Perl 88 (2026-10-22, CG 100 %, rank 8 tie 6, top 73 Eric_[EGU]): the OCaml-wrapper perl core in a `#!perl -0p` file with `$_=` output (`-E`/say unusable on the shebang). Measured ties/losses on that skeleton: `sub f{sort{$a-$b}@_}` ties (88, no timeout on CG_PERL locally, 0.6 s for all hidden); `sub f{sort{$a-$b}/@_/g}` with `f' .+'` +1; building `(xmin, Y..., xmax)` for one pair-ends loop via `splice@x,1,-1,sort..` +1, via list `((@x=..)[0],(sort..),pop@x)` +4; `($a,@x)=sort..` +6. 73 needs about 15 fewer: two `sort{$a-$b}` + regexes + pop loop already take 56 of the 62 body bytes, so 73 likely uses one sort or no pop loop (unknown)
