# Regex alternation as a counting DP (TRIE)

Verified on CG (the-resistance perl 165, 100%, 2026-10-22).

Count ways to split a string into dictionary words without a hash or a length window: build `$r` = words joined by `|` (duplicates kept), then

```
@d=1;$s=~/(?{$d[pos]})($r)(?{$d[pos]+=$^R})\D/
```

- the engine tries start positions in ascending order, so `d[i]` is final when position i is entered; `(?{$d[pos]})` stores it in `$^R`, the second block adds it at each end position, then a never-matching atom backtracks through every alternative
- perl compiles the alternation to a TRIE: all accepting words at a position are enumerated, duplicates counted separately
- `\D` (2 bytes) as fail atom when the subject is digits only; a literal like `2` gets pruned by the optimizer (required substring missing, no block runs). `(*F)` 4 bytes otherwise. `$` / `^` stop early on a real match
- empty alternatives (leading/trailing `|`, an input line that converts to "") multiply every `d[i]` after use: k empty alts give `d[L]=(k+1)*true`, divide at the end; `d[L]` is then always defined, so `$d[-1]` is safe even for answer 0. `/k` goes NV above 2^63/k
- TRAP: TRIE is dropped (plain BRANCH, O(N) alternatives per position, 100x slower) past a state limit: ~9000 words of 5-6 letters (Morse codes) still fit, 6000 words of ~10 letters do not (`perl -Mre=debug`, grep TRIE)
- empty-alt counting (measured 2026-10-22, perl 5.34): leading empties (before the first nonempty alt) count one each, empties inside the trie count ONCE total: `|0|1|` 3x, `||0|1` 3x, `0||1|` 2x, `xx||0|1|` 2x. Put input lines that must vanish FIRST (they become leading empties), divide by (leading + 1 if a trailing `|`) + 1
- a long junk alternative (e.g. 5L symbols from an unconverted Morse line) blows the TRIE state limit: test 04 2.08 s vs 0.04 s; convert junk lines to "" instead

- the never-matching tail that forces backtracking must not be a literal: `/(?{..})($r)(?{..})-/` on a 0/1 subject prints nothing computed, the optimizer rejects the match on the missing required substring `-` before any code block runs. `\D` (2) or `(*F)` (4) work (verified 2026-10-22, the-resistance)

## Shorter form: `$d[$-[0]]`, one trailing empty (verified on CG, the-resistance perl 150, 100%, 2026-10-22)

```
@d=$r=~/\|+/;$`=~/($')(?{$d[pos]+=$d[$-[0]]})\D/;$_=$d[-1]/2
```

- `$-[0]` inside the code block = start of the current attempt, so no `(?{$d[pos]})`/`$^R` pair (-6)
- only valid when the ONLY empty alternative is the trailing one: the TRIE tries alternatives in order, so the trailing empty doubles `d[i]` after every word from i used it; only `d[L]` (start position L is visited) ends doubled, `/2`. A leading empty would double `d[i]` before use: path-dependent factor, wrong
- one string holds subject and words: `$r` = `M|<empties>|w1|..|wn|`; `@d=$r=~/\|+/` splits it (list-context match without groups returns (1) = `@d=1`), `$\`` = subject, `$'` = alternation (interpolated at compile time, before the match changes anything)
