# The Resistance

pretty_id `the-resistance-codesize`, io.

## Rules (as referee)

- input: Morse line (`.` and `-` only), `N`, then N uppercase words, one per line
- output: number of ways to split the whole Morse line into dictionary words, as integer; exact match of line 1
- distinct words with the same Morse encoding count separately (test 05: CAT/KIM/TEXT/TREM/CEM, answer 125): a set of encodings undercounts
- R < 2^63 by statement: needs 64-bit ints, not 32 (test 04 = 57330892800, test 06 = 2971215073)
- words up to 19 letters, so a match spans at most 76 Morse symbols

## Tests

- 01-03 tiny, 04: L 9884, N 9444, words mostly 5-6 letters, answer 57330892800
- 05 same-encoding words, 06: 52 dots with dictionary `E I` (Fibonacci, 2971215073)
- io limit 6000 ms per test (shared probe `knowledge/lang/_shared/io-time-limit.md`, not re-probed here)

## Traps

- tests 05 and 06 have NO final newline on the last word (captured byte-exact through lzma+b85); 01-04 end with `\n`. A per-line reader that needs `\n` loses the last word; `split()` is safe
- `capture-io.sh` truncates test 04 (CG output cap ~4 kB): captured as lzma+b85 slices, 12 games (journal `referee.the-resistance.*`, scratch `capture.sh`)
- naive O(L * 76) substring dict lookups are fine for test 04 in Python (reference with a trie: 0.08 s on 04, 0.9 s on validator L 99029 N 40000)
- validators (ours) cover: R exactly near 2^63 (Fibonacci 91 dots, dashes, alphabet), R = 0 with trailing unmatched symbol, single letter without final newline, L ~ 99000 with N 40000

## Shortest known

General method (verified, Python 264 bytes, 18/18 local tests + validators, 2026-10-20):

- Morse of a letter from the binary-heap table `"  ETIANMSURWDKGOHVF L PJBXCYZQ"`: index of the letter (2..29) in binary, drop the leading `1` bit, `0` = dot, `1` = dash (checked for all 26 letters). Positions 0, 1, 19, 21 are fillers, any non-letter char. Python `bin(T.find(c))[3:]`; other langs: `sprintf"%b"` / `to_s(2)` / `toString(2)` + drop first char
- translate the Morse line instead of each code: `.`->`0`, `-`->`1` once (`tr`/`y/.-/01/`; Python: `.translate({45:49,46:48})` on the whole input before `split`, digits of N untouched). Dash = 45 odd, dot = 46 even, so `ord%2` also gives the bit
- word codes go into a count map (code -> number of words), NOT a set: same-encoding words multiply (test 05)
- forward DP: `d[0]=1`, for each position i and k in 0..76: `d[i+k] += d[i] * count[s[i:i+k]]`, answer `d[L]`. k=0 is the empty slice, never a key, so `range(77)` is fine. Slices past the end truncate: they add only to indices > L (pad d, Python `[1]+[0]*9**6`), d[L] stays exact
- N (line 2) is ignorable: read all tokens with `split`, which also survives the missing final newline of tests 05/06
- integers: answers reach ~2^63 on our validators (05, 06, 08, 11); CG tests max 57330892800 (> 2^32, < 2^53). 64-bit needed; JS/TS/PHP floats lose exactness above 2^53 on our validators (unknown whether hidden CG validators go that high)
- time: Python window-77 DP 4.9 s on validators/10 (L 99029, N 40000), limit 6 s; 0.44 s on test 04. Tight but passes; a slower lang may need `if d[i]` skip

Rejected:

- `Counter` import: 9 bytes longer than `D[k]=D.get(k,0)+1`
- `list.count` per slice instead of a map: O(L*77*N), far over the limit on test 04
- regex enumeration of splits (`(?{$c++})(*FAIL)`): exponential, answers up to 2^63
- full Morse table as strings (~80 chars): heap table is 30

Leaderboard gap (unverified): tops ruby 57, perl 65, php 67, python 70, bash 72 are below the table + DP size (table alone 32 bytes with quotes). They almost surely hardcode answers keyed on the hidden CG validators (same pattern as `dwarfs-standing-on-the-shoulders-of-giants`, `super-computer`). Local validators are synthetic and reject hardcodes; matching needs the hidden validator inputs or the user's go-ahead to probe CG by submit.

- ours Python 991 (plain reference, 2026-10-19), rank #259 tie

- bash 219 (perl -0ae wrapper, 18/18 local, CG rank #4 at submit): `($s,$_,@F)=@F` drops morse and N in one list assign, `for(@F){$h{join"",map{...}/./g}++}` needs no loop var, `@d=1`, window `1..76`
