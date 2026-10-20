# Bash

- [env](env.md): bash 5.1.16, UTF-8 locale, coreutils 8.32 + sed 4.2.2 + grep 2.27 + mawk 1.3.3 + bc + perl + python3 on PATH, no dc/gawk, local tools newer, verified
- [perl-wrapper](perl-wrapper.md): `perl -pe` one-liner as bash solution, -p eats line 1 and prints `$_` counter, `-lnE` + `say` for line loops, verified
- [perl-slurp-fields](perl-slurp-fields.md): `perl -0aE` slurp + `@F` for float one-shots, list index `//` cap, mawk `x++/2` regex trap, verified
- [subshell-recursion](subshell-recursion.md): `f()(...)` recursion with free locals, min via `f|sort -n|sed q`, leaf flag from a loop var + `k= f` reset, verified
- [read-in-order](read-in-order.md): `read f e[f] a` binds names in order, subscript sees the f just read; throwaway last name, verified
- [missing-command-print](missing-command-print.md): `${d:+echo} $out` prints only if d set, else not-found on stderr; expansion arith error aborts whole loop, verified
- [word-as-arith-var](word-as-arith-var.md): `LEFT=-1` then `d=LEFT` is -1 in arithmetic; `(x^s)>s` sign test, verified
- [perl-flood-worklist](perl-flood-worklist.md): perl -0777nE grid flood with growing `for@k` list, 4-arg substr mark+test, `~-@k/4` count, verified
- [subshell-loop](subshell-loop.md): `f()(BODY;f);f` endless loop, 2-4 bytes under `for((;;)){ }`; unset/empty var in `(($c=..,1))` as trigger, verified
- [awk-big-number-print](awk-big-number-print.md): mawk print of >=2^31 gives exponent form, use printf"%.0f"; n from header as sign weight beats array+END, verified
- [perl-sort-slurp-min](perl-sort-slurp-min.md): `perl -0aE` slurp, shift N, sort comparator min, `say 0+(..)[0]` empty fallback; abs precedence trap, verified
- [perl-column-stream](perl-column-stream.md): perl -0ne one-liner tricks: s///ge RLE transpose, \31 octal in regex, bool index, float % chr, verified on music-scores
- [source-self-loop](source-self-loop.md): `;. $0` endless loop for 5 bytes, 4 under `f()(..;f);f`, verified on CG
- [pipeline-argmax](pipeline-argmax.md): `sed 8q|nl -v0 -w1|rev|sort|tail -c2` argmax of one-char lines, `split --filter` lazy trap, verified
- [init-slurp-head](init-slurp-head.md): `a=($(head -4))` swallows init lines + first turn line of an interactive puzzle in 15 bytes, verified
- [response-word-as-arith-var](response-word-as-arith-var.md): `COLDER=1` makes `r` ("WARMER"/"COLDER") a 0/1 in `(( ))`, `r^q>t` precedence, ternary-else parens, `--s/2`, verified
- [loop-body-function-stale](loop-body-function-stale.md): `e(){ echo;read;update;}` reused for loop and final jump, stale update harmless, verified
- [perl-regex-bfs](perl-regex-bfs.md): regex flood BFS with plain marks, direction from `$#-`, tr-list phase state, 5.34 variable lookbehind bug, verified
