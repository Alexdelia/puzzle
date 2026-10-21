# eval(c+"=_"+c): letter-named bounds, mids stored in `_`-prefixed vars

Verified locally 2026-10-21 (shadows-of-the-knight-episode-1, 16/16): `print(_L=_R=+L+R>>1,_U=_D=+U+D>>1)` then `for(c of readline())eval(c+"=_"+c)`. Beats the packed `p=x+y*M` form by 14 B (no `M`, `%`, `/`). `+L` needed (L is a string from split), `R=D=0` needed (undeclared read throws, `~~R` fails).
