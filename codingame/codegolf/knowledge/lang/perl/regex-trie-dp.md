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
