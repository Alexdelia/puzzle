# local $_ as recursion state

A recursive sub whose helpers read `$_` can take its main argument as `local($_,@r,%h)=@_` instead of `my($i,@r,%h)=@_`: helper calls become bare `S` instead of `S$_=$i` (-4 each, +3 for `local`). `for`/`map` inside the body localize `$_` and restore it, so `$_` is still the argument after them and after each recursive call returns (the callee's `local` is undone). Verified on CG: the-fall-episode-3 perl 600 -> 584 chain (2026-10-22).

Related: `f(S||next,...)` inside a `for` body calls the child only when the helper succeeds; `next` is legal inside an argument list.
