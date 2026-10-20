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
