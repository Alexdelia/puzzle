# Super Computer

pretty_id `super-computer-codesize`, leaderboard `super-computer-codesize`, io.

## Rules (as referee)

- input: `N`, then `N` lines `J D` (start day, duration). Output: max count of pairwise non-overlapping calculations
- calculation covers days `J .. J+D-1` inclusive: two are compatible iff `J1+D1 <= J2`. Strict `<` is wrong (test 2 gives 3 instead of 4; test 3 gives 7829 instead of 7879)
- classic interval scheduling: sort by end `J+D`, take greedily
- io limit 6000 ms per test (shared rule, `lang/_shared/io-time-limit.md`, not re-probed); plain Python greedy with sort takes 170 ms on N=99999 locally

## Tests (CG, probed 2026-10-19)

- 01, 02 = statement examples, answers 3 and 4
- 03 "Large number of scientists": N=99999, 1078215 chars, J in 1..999988, D in 1..999 (all 999 values present), unsorted, uniform random, 99992 distinct pairs (duplicate lines exist), 95225 distinct J, answer 7879
- 03 is too big to capture (CG output cap ~4 kB, 580 kB lzma). Local `tests/03.in` is synthetic: same N and distribution (Python `random.Random(0)`), answer 7978. CG 03 sha256 `8dce459ea33cb6e4f7e026a6efed9c070896fe6697ece3d15881a0add5f8f594`
- end day `J+D` reaches up to ~1000997: above 1e6, fits int32

## Input quirks

- final newline present (all 3 tests), no trailing spaces, no CR, single space separator
- N = number of following lines: reading all lines after the first works

## Validators (local)

- edge cases: N=1, identical intervals, touching chain (`J+D == next J`, all fit), overlap by one day, nested long interval, many ties, ends past 1e6, N=99999 all disjoint (answer = N), heavy overlap

## Shortest known

Method: greedy by latest start (mirror of earliest-end greedy). Verified 15/15 local (tests + validators) in bash, perl, ruby, python, c; bash version passed CG test 03 (2026-10-19), so `sort`, `awk`, `wc` exist on CG.

- sort intervals by start `J` descending, ties in any order (correct: future choices depend only on the chosen start, same start = same state)
- `e` = start of last taken, `0` = none yet. Take iff not (`e` and `J+D > e`), then `e = J`. Answer = number taken
- `e=0` sentinel works because `J > 0`. No infinity constant, no end key to compute before sorting
- why short: sorts by start only, and "numeric value of the line" = `J` in languages that read the leading number of `"J D"` (awk/`sort -n`, perl `$b-$a`, ruby `to_i`, php, js `parseInt`): no field split before the sort
- the `N` line must be dropped (`read;` in shell, `<>;`/`gets` in perl/ruby). Kept, it acts as a zero-length interval at day `N` and moves `e`: validators 12 fails by +1
- shell form: `sort -nr`, then awk with only a pattern (prints taken lines), count with `wc -l`. Pattern `$1+$2>e&&e?0:e=$1` (17 chars): ternary in pattern, assignment yields `J > 0` = truthy. Shorter than `{...;c++}END{print c}` by 8
- ruby/perl count with `count{...}` / `~~grep{...}` over the sorted lines, same ternary
- shell core 43 bytes (bash top 45). System wrapper cells (`lang/_shared/system-wrapper.md`): core + wrapper reached locally c 62 `main(){system("...");}` (top 64), ruby 49 `exec"..."`, perl 51 `exec q(...)`, python 66 `os.system`
- native reached: perl 55, ruby 81, python 108 (end-ascending with `key=sum` on `[J,D]` lists beats start-descending `[::-1]` + `9e9`/sentinel in python)

Alternative, no sort (for languages where sort with comparator is expensive): bucket by end. `m[J+D] = max J`, sweep `t` ascending over `2^21` days with `e=1` (starts >= 1 so empty bucket `0` never passes `m[t]>=e`), take iff `m[t]>=e`, then `e=t`. Verified python 139, c 142; longer than the wrapper in c.

Rejected:

- strict `J+D < e` / `>` off-by-one: wrong (see Rules)
- replacement greedy on ascending start (`sort -n`, keep `min` end when overlapping, no sentinel needed): awk 55 vs 43
- precomputing end key for `sort -n` in shell: extra awk stage, longer
- tops below any general method (c 64 tie only via wrapper; lua 58, ocaml 59, python 59, perl/ruby 39, php 40) are likely hardcodes keyed on the hidden CG validators (unverified, same pattern as `dwarfs-standing-on-the-shoulders-of-giants`). Local validators are synthetic and reject hardcodes; matching them needs the user's go-ahead to probe CG
