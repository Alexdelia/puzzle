# Dwarfs standing on the shoulders of giants

pretty_id `dwarfs-standing-on-the-shoulders-of-giants-codesize`, io.

## Rules (as referee)

- input: line `N`, then `N` lines `X Y` (X influences Y), any order
- graph is a DAG (no cycle, no self edge), may be disconnected, several roots
- output: number of nodes (not edges) on the longest path, single line
- constraints: `0 < N < 10000`, `0 < X, Y < 10000`

## Tests (captured 2026-10-19)

- 4 tests, 3, 8, 4 and 9 edges, ids 1..11, answers 3 4 3 5
- tests 01-03 are the statement examples; 04 "Several mentors 2" is an extra small case
- single space between X and Y, one final `\n`, no trailing space, no blank lines
- local validators 01-06: single edge with X > Y (answer 2), ids up to 9999, longest vs shortest path trap, several components, layered DAG with skip edges (39 edges), random DAG (19 edges, answer 7)

## Timing

- io limit 6000 ms per test, from `knowledge/lang/_shared/io-time-limit.md`, not re-probed here
- sizes in tests are tiny: plain recursive DFS without memo is fast on every local case

## Unverified

- hidden validator sizes unknown; local validators stay under 40 edges and depth 7, so a deep-recursion solution passes locally, risk only if CG hides a long chain

## Shortest known

### Method: peel roots, count passes

- repeat: keep only edges `X Y` whose `X` is some edge's `Y` (drop edges leaving a root); count passes until no edge left
- answer = passes + 1. Proof: each pass shortens the longest path by exactly its first edge (every later edge keeps an incoming edge, any surviving path extends one edge back)
- no node map, no max, no recursion: state is the edge list (or the raw lines) only
- text form: keep line `X Y\n` iff the joined text contains ` X\n`. Leading space needed (` 3\n` must not match `13\n`), trailing newline relies on the final `\n` the tests have
- the `N` line can stay in the list: it has no space, never matches, falls out in pass 1 without adding a pass. Skips the read-and-discard of line 1
- needs: filter/grep over a list, string contains (or regex with interpolation), join

### Lengths reached (local referee, tests + validators pass)

- perl 52 (53 with `\n`, raw newline saves 1): loop `$i++while@e=grep{/ /;...}@e`, print `$i+2`; `/ /;` sets `$`` to X before the target test (system perl by hand, tests + validators pass)
- bash 57 (CG accepted, 2026-10-20, raw newline in the regex instead of `\n`): perl one-liner under `-p`: `$_=2;$_++while...` counts in `$_` (grep restores `$_`), `-p` prints it, line 1 eaten by `-p` so `@e` holds edges only. Pure bash grep peel (`grep -wf` with `^Y` patterns, `-w` keeps `^1` off `13`) = 85
- perl 57: grep on raw lines, `"@e"=~/ $`\n/` as the target test, `N` line kept (run with system perl 5.42 by hand: no `CG_PERL` in this shell, run.sh refused)
- python 93: `_,*e=map(str.split,open(0))`, filter `p[0]in[*zip(*e)][1]` (zip column of targets), while loop with counter
- python 99 (any) / 105 (recursive `f=lambda x:1+max([f(b)for a,b in e if a==x]+[0])`): longer

### Ideas, untested

- ruby: `e.rassoc(a)` finds an edge whose `Y` is `a`: `e=e.select{|a,|e.rassoc a}` per pass
- regex-only peel on one string (lookahead over the doubled text): not tried, likely longer than grep

### Rejected

- DFS recursion: needs a max with empty default, longer in every lang tried
- Bellman-Ford relaxation (`d[y]=max(d[y],d[x]+1)` N times): needs node map, max and a final max: longer than peel
- `N`-only formula: local validators map `N=4` to both 3 and 5, so impossible locally. On the 4 CG tests alone `(n*n+1)//33+3` fits (no `n*a%b+c` form fits)

### Leaderboard gap (unverified)

- tops ruby 20, perl 21, php 21, bash 27, python 47 (all Icebox / Eric_[EGU]) sit far below any general method (perl general 57). They almost surely hardcode the answer from a property of the input (N, length, checksum) tuned to the hidden CG validators
- local validators are synthetic and reject such hardcodes, so `submit.sh` refuses them. Matching those tops needs the hidden validator inputs or the user's go-ahead to probe CG by submit: user decision, not a cell worker's
