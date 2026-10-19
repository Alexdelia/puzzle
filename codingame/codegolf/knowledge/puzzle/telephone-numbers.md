# Telephone Numbers

pretty_id `telephone-numbers-codesize`, leaderboard `telephone-numbers-codesize`, io.

## Rules (as referee)

- line 1 `N`, then `N` lines of digits; answer = number of distinct non-empty prefixes over all numbers (trie node count without root)
- duplicate numbers count once; a number that is a prefix of another adds nothing
- statement: `0 <= N <= 10000`, `2 <= L <= 20`; N=0 never seen in tests, unknown in validators (not in our validators)
- io limit 6000 ms per test (shared rule, `lang/_shared/io-time-limit.md`, not re-probed); plain Python set of prefixes takes 60 ms on test 5

## Input (captured 2026-10-19)

- final newline present in all 5 tests, no CR, no spaces, digits only
- tests 1-4 are the statement examples (answers 10, 20, 10, 28)
- test 5: N=10000, lengths 3..14 (5320 of length 10, none above 14 despite L <= 20), 84 duplicate lines, answer 45512
- `capture-io.sh` truncates test 5 (92778 chars, CG stderr cap ~4 kB): `referee/capture/capture.sh 5` captures lzma+b85 in 3800-char chunks, 14 games

## Validators (ours)

- mirrors of tests 1-4 with other values, a long list with test 5 length histogram, 10000 numbers of length 20, 50 copies of one 2-digit number, all 100 2-digit numbers plus 3-digit ones, shuffled prefix chain of lengths 2..20, many numbers with leading `0`/`00`
- numbers must stay strings: leading zeros matter (`0123` and `123` differ), int conversion is wrong

## Shortest known

Method (2026-10-19, algorithm worker): answer = size of the set of distinct non-empty prefixes of lines 2..N+1. N itself is never needed, only its line skipped (its digits must not enter the set: `10000` would add `1`, `10`, ...). Dedup of numbers and prefix-of-another fall out of the set for free.

Ways to produce the prefixes, pick by language feature:

1. slices in a nested comprehension over lines that keep their `\n`: `s[:~i]` for `i` in `0..20` gives the number (`s[:-1]`), every shorter prefix, and always `''` (`s[:-21]`, line is at most 21 chars), so print `size-1`. Python 64, all local pass. Relies on the final newline (present in every test and validator, last line included)
2. chop loop: repeat "emit, drop last char" until one char is left. sed: `'1d;:a;p;s/\B.$//;ta'|sort -u|wc -l` (bash 39, system sed, all local pass): `\B.$` refuses to remove the only char, so no empty line is ever printed and the final autoprint is a harmless duplicate. Ruby `chop!` / Perl `chop` same idea, unverified (Perl: `chop` returns the removed char, `"0"` is false, do not use it as loop condition)
3. regex backtracking with a code callout on `^.+` (Perl `(?{...})` then a failing token) enumerates all prefixes of a line; unverified
4. no set: sort the lines, answer = sum over sorted lines of `len(s) - lcp(s, previous)`; duplicates and prefix lines give 0. For languages with sort but no cheap set; unverified, not measured
5. integer key: prefix -> `x = x*10 + d` started at 1 (keeps leading zeros apart), reset at `\n`, mark in a big array/bitmap (`x % M`, collisions possible), count first marks. For C-like; unverified

Lengths reached (local referee, 15/15 tests + validators):

- Python 64: `_,*a=open(0)` + set comprehension of method 1 + `-1`
- Python 67: `s[:i]for i in range(1,len(s))`, no `-1` (also needs the final newline)
- Python 74: `open(0).read().split()[1:]` + `range(21)` + `-1`
- bash 39 (method 2), 41 with `s/.$//` + `grep -c .` (empty line filtered at count), 42 with `-n`

Gaps not explained: Python top 47 (17 under ours), bash top 33 (6 under), Perl/Ruby 31. A shorter prefix generation or a different count must exist; the 47 cannot be a pair comprehension (`print(len({...}))` + `open(0)` + the shortest `s[:i]for s in a for i in range(..)` already exceed it).

Rejected:

- Python `input()` then `open(0)`: prints 0, `input()` drains stdin into its buffer
- `s[:-i]` for `i` in `1..20`, `-1`: no `''` when every number has length 20 (validator 10), off by one
- char stream walrus `{p:=p*(c>'\n')+c for c in open(0).read()}`: the N line enters the set, its length is unknown; per-line version needs `p='\n'` init, longer than slices
- counting suffixes instead (`s/.//`, `rev`): suffix count differs from prefix count
