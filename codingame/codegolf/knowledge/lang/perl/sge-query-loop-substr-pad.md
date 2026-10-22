# s///ge query loop, header-numify width, substr pad with negative wrap

Verified surface (OCaml wrapper around a `perl -0nE` core, local 19/19, CG submit 146 rank 1, 2026-10-22).

- `s/(.+) (.+)/CODE;CODE;say.../ge` replaces `while(/(.+) (.+)/g){...}`: 6 B less. The replacement is plain code, `;` allowed, inner matches (`=~O`) and `for` loops inside do not break the outer `/g` walk. Read `$1`/`$2` before any inner match
- `$w=s/.+\n.+\n//+$&` drops the two header lines `L\nH\n` and sets `w=L+1`: `s///` returns 1, `$&` numifies to its first line `L`. 7 B under `s/(.+)\n.*\n/1x($w=$1+1)/e`
- grid as string, 4-arg `substr($t,$i,1,X)` as test-and-mark: a negative offset counts from the end, so one pad of `w` non-target chars at the end (`$t=$_.1x$w`) serves both the down overflow of the last row and the up overflow of row 0 (index `-w..-1`). No top pad, no `+1` row shift: start `$2*$w+$1`
