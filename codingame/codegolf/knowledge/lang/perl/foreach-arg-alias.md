# `for($_[0]){...}` as the sub's `$_`

In a recursive sub that works on its first argument through `$_` (step function `S` reading `$_`), wrap the body in `for($_[0]){...}` instead of `my($i,...)=@_` + `$_=$i;` before each use: foreach aliases `$_` and restores it when the loop ends, so a nested call (which rebinds `$_` the same way) leaves the caller's `$_` intact after it returns. `return` inside the loop returns from the sub. Inner `map`/`grep`/`for` modifiers also restore `$_`.

- `sub f{my%h;...;for($_[0]){S 1;...;for$c(...){local@z=@z;R;S&&$k==$X||f(map{S}@_)and return$c}}}`: 14 under `my($i,@r,%h)=@_;...$_=$i;...$_=$i;...`
- bonus: `S$_` passes `$_` as `$_[0]` of `S`; a nonzero state then doubles as a truthy flag argument (`S 1` and `S$_` call the same "no accept check" variant)
- `map{S}@_` (no args) for the flagless call: a bare predeclared sub name before `}` gets an empty list

Precedence used with it: comparisons bind tighter than `|`, `|` tighter than `&&`, so `$d&&$E[..]|$_[0]&&$k*4+$d` needs no parens (2 under `(9>$E[..]|$_[0])` with the parens).

Verified: the-fall-episode-2 perl 598 -> 586 chain, local 26/26, CG 100% at 592 and 586 (2026-10-22).
