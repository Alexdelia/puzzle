# perl -0aE for float work in bash

Verified local (bash 5.1.16 + perl 5.34.1) and run on CG (blunder-episode-3, perl 5.34.0 executed, output judged).

- bash has no floats; `perl -0aE'...'` beats awk for numeric one-shots: `-0` slurps stdin, `-a` splits all whitespace tokens into `@F` (and implies `-n` since perl 5.20, so no `n` flag), `$F[-1]`, `$F[-2]` give last tokens
- list literal with barewords: `(1,n,"n^2")[$i]`; float index truncates; out of range gives undef, cap with `//"x"`
- assignment inside a list reuses a string: `(1,$l="log n",n,"n $l")`, left to right
- `pop@F/$F[2]` is a compile error (pop takes `@F/...` as its argument), keep `$F[-1]`
- mawk: `x++/2` is parsed as the start of a regex (runaway regular expression); write `x++*.5`
- mawk: `$(k)` with float `k` truncates; assigning `$0` after `FS=","` resplits with the new FS
