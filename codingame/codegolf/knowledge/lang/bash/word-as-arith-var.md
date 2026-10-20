# input word as arithmetic variable

Verified locally (bash 5.1.16) and on CG (dont-panic submission accepted 2026-10-20).

- in `$[..]`, `((..))` or a subscript, a bare name whose value is a word (`d=LEFT`) evaluates the variable of that name, unset = 0. Set only the cases that matter: `LEFT=-1`, then `d` is 0 for `RIGHT`/`NONE`/empty, -1 for `LEFT`
- with `s` in {0,-1}: `(x^s)>s` is `x>0` for `s=0` and `x<0` for `s=-1` (xor with -1 is `-x-1`); shorter than `x*(2*${#d}-9)>0`
