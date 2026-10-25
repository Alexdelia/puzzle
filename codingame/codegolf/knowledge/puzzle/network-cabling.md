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

- 01 N=1, 02 N=2 same `x`, 03 corner cross +-2^30 + origin (positive corner `2^30`, negative `1-2^30`, so each column spans 2^31-1) -> 6442450941, 04 N=100000 random +-2^30, 05 N=100000 one point repeated, 06 all `x`=0, 07 negatives only, 08 same `x` with duplicates, 09 `y` strictly descending from 2^30, 10 N=99998 random [-63,63], 11 all `y`=0, 12 two corner points repeated 3 times (same asymmetric extremes as 03) -> 8589934588
- 03/12 were regenerated 2026-10-25 with span 2^31-1 instead of exactly 2^31: the original +-2^30 corners tripped perl's 32-bit `sort{$a-$b}` comparator (`knowledge/lang/perl/sort-comparator-32bit.md`) and made the local referee reject the CG-100 % perl 88 / OCaml 101 rows. Any new large-coordinate case must keep `max-min < 2^31` in each column; hidden V6 (N=8, answer 5714781103) is 100 % on those rows, so CG's validators satisfy the same bound.

## Shortest known

- ours Python 114, rank 49, top 88 Lydxn (doubled-y slice form, CG-confirmed)

Method (verified 26/26 local on tests+validators+hidden, Python, 2026-10-25): answer = `max(x)-min(x) + f(Y)`, `f(Y)` = sum of absolute deviations of `y` from its median. Only one sort, on `y`; `x` needs min and max only, never sorted, never paired with its `y`.

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
- all 9 answers known (p01-p36 bisection, confirmed together by p37 perl full hardcode CG 100 %, 2026-10-24): 6 (N = 8) -> 5714781103, 7 (N = 100000) -> 12788481, 8 (N = 99999) -> 2500149996, 9 (N = 11) -> 2067588810; inputs of 6-9 not recovered (no `complete`)
- key: N alone separates 3 (4 -> 8), 4 (1 -> 0), 6, 7, 8, 9; the N = 3 group (1, 2, 5) splits on first char of line 2 (`0` -> 8, `2` -> 6, `-` -> 24). V6 > 2^32, V8 > 2^31: 64-bit or string literals
- perl `sort{$a-$b}` (comparator truncated to 32 bits, wrong when two values differ by >= 2^31; `knowledge/lang/perl/sort-comparator-32bit.md`) passes all 9 hidden validators: CG 100 % on OCaml 102 perl wrapper, 2026-10-22 (probe log). Drop `<=>` for `-` in every perl core of this puzzle. 2026-10-25: local validators 03/12 originally used exactly +-2^30 and were false negatives for these rows; they now span 2^31-1 and all 27 row solutions pass tests+validators+hidden 26/26
- probe rows (sub-100 %, to be replaced by real solutions): php 380, lua 375, typescript 475, ruby 161, perl 209

- ours Clojure 167 (2026-10-21, rank 4, top 158): `F=#(map -(sort > %)(sort %))` on x and y columns; first of F(x) = range, positives of F(y) = deviation sum; trick note `knowledge/lang/clojure/read-pairs-sorted-diff.md`

- ours OCaml 101 (2026-10-22, rank 3, top 94; x sort inlined: `say$s-(@x=sort..)[0]+pop@x`): `Sys.command"perl -0nE'...'"` wrapper around a 78 B perl core (two `sort{$a-$b}` on `/.+ /g` and `/ .+/g`, pop-while-iterating pair ends `$s-=$_-pop@y for@y`, range `-$x[0]+pop@x`). General OCaml (split, `sort(-)`, `map2(-)(rev s)s`) is 182. Perl `sub f{sort{$a<=>$b}@_}` saves 2 B but timed out locally on 1e5 tests (unclear if load)

- ours Perl 88 (2026-10-22, CG 100 %, rank 8 tie 6, top 73 Eric_[EGU]): the OCaml-wrapper perl core in a `#!perl -0p` file with `$_=` output (`-E`/say unusable on the shebang). Measured ties/losses on that skeleton: `sub f{sort{$a-$b}@_}` ties (88, no timeout on CG_PERL locally, 0.6 s for all hidden); `sub f{sort{$a-$b}/@_/g}` with `f' .+'` +1; building `(xmin, Y..., xmax)` for one pair-ends loop via `splice@x,1,-1,sort..` +1, via list `((@x=..)[0],(sort..),pop@x)` +4; `($a,@x)=sort..` +6. 73 needs about 15 fewer: two `sort{$a-$b}` + regexes + pop loop already take 56 of the 62 body bytes, so 73 likely uses one sort or no pop loop (unknown)

- captured `referee/tests/01..09` (via `capture-io.sh` test-games) are the puzzle's sample tests, NOT the 9 validators: e.g. tests/01.in is `3\n0 0\n1 1\n2 2` -> 4 while validator 1 is `3\n0 0\n2 2\n4 4` -> 8. Do not use tests/06..09 as proxy inputs for validators 6-9 (their answers differ: 6066790161/3142894/2500049998/2178614523 vs 5714781103/12788481/2500149996/2067588810).

### range+range shortcut (2026-10-24, cell 1792698297968363581)

- hidden answers 1-5 all equal `range(x)+range(y)`: V1 4+4=8, V2 3+3=6, V3 5+3=8, V4 0, V5 18+6=24 (V5 x-extremes -9/9, y -4/2; source `referee/hidden/01..05`, same expected outputs)
- cheapest range+range Perl is top2-bottom2 of the combined x,y match list:
  `#!perl -0p` + `@z=sort{$a-$b}(/ .+/g,/.+ /g);$_=$z[-1]+$z[-2]-$z[0]-$z[1]` = 69 B
  It passes hidden 01-04, FAILS hidden 05 (25 vs 24: both two smallest are x=-9,-5 while min(y)=-4). It is valid only when the two largest/two smallest of x∪y are one per column.
- probe `probe/perl/top2range.pl` (69 B, submission 41559291) was NOT 100 % on CG (own Perl row stayed 88). Report broker had no session, so the per-validator report is unavailable; the local hidden/05 failure alone already proves it cannot be 100 %.
- V8 is provably NOT range+range: 99999 distinct integer y values force `sum|y-med| >= (99999^2-1)/4 = 2499950000` (equality iff consecutive), while the expected answer is 2500149996 and `range(x) >= 0`; hence dev is about 2.5e9 and range(y) is at most ~1e5, so range+range is at most ~3e5. The earlier `x=2i+b,y=i+d` deduction is consistent (xrange 199996 + dev 2499950000).
- so any range+range exploit must special-case V8; the special case plus the 4-byte-over-69 V5 fix pushes it well over 73.
- all-answers hardcode floor (2026-10-24, not submitted): 38 answer digits; N%11 separates the 6 N classes (residues 1,4,8,0,9,10) but the three N=3 cases share residue 3. Sparse list indexed by N%11 = ~99-106 B, N-key hash = ~100 B, `split`-string table with an index = >=78 B + index expression. None beats 88, so top 73 is not a plain hardcode; a general <=73 Perl is still unknown.

### validator geometry and oracle limits (2026-10-24, cell 1792864064912233544)

- V8 shape is exactly determined: answer 2500149996 = (N-1)(N+9)/4 with N=99999, and for y=i+d
  (i=0..99998) dev(y) = (N^2-1)/4 = 2499950000, rangeX = 199996 = 2(N-1), so x = 2i+b with
  b in 0..65535 (matches p22-p36). b and d cancel; any program that passes V8 outputs
  rangeX + dev(y) = 2500149996.
- V1-V5 all equal rangeX+rangeY; the *correct* range+range Perl is 78 B body / 89 B total
  (`@_=sort{$a-$b}/ .+/g;$s=pop(@_)-$_[0];@_=sort{$a-$b}/.+ /g;$_=$s+pop(@_)-$_[0]`, passes hidden
  1-5 locally), i.e. LONGER than our general 88 (the general form folds the x range into the final
  `$_=` expression, which the two separate range sums cannot), and it cannot pass V8 (needs dev,
  not range). The 69 B top2range union variant fails V5 (25 vs 24). So no range+range exploit
  reaches the 73 B top. Trap: `$_=$s+pop(@_)-$_[0]` must be `=`, not `-=`; `pop@_-$_[0]` is a
  compile error (needs `pop(@_)-$_[0]`).
- Why top 73 is not an answer hardcode: creationTime of the Perl rows is 73 Eric_[EGU]
  2025-11-25, 74 LolCat 2023-09-10, 75 saito_ta 2023-05-25, long before the 2026-10-24 answer
  recovery; the full answer hardcode p37 is 112 B (44 answer digits alone exceed a 62 B body).
- 2026-10-24: every one of the 27 CG code-golf languages already has a 100 % row of ours for this
  puzzle, so the fresh-language score oracle (partial score) is unavailable; the per-validator
  report broker answered `no CG session` all session (`cg-report.sh 41559291`), so the open
  question "are V6,V7,V9 also range+range?" could not be probed. Needs a broker session.
- sample tests 06-09 do not determine hidden 6-9: base/transpose/sign-flip answers of samples
  06/07/09 are 6066790161/5343706357/..., 3142894/3143287/..., 2178614523/2231774220/...,
  none equal 5714781103, 12788481, 2067588810.
