# interactive golf idioms (verified on shadows-of-the-knight-episode-2, local)

- `z,_,a=4.times.map{gets.split.map &:to_i}` reads 3 init lines + UNKNOWN line
- response compare: `gets;/S/?x:y` (bare regex matches `$_`), `$_>?W` true only for "WARMER" (longer string > "W")
- `q>t==$_>?W` parses as `(q>t)==($_>?W)`
- `c=t+a[i]=q` assigns and sums in one go
- `h=c-1>>1` instead of `(c-1)/2`
- `$>.sync=1` mandatory: without it the referee times out on turn 1

Parity-bisection refinements (shadows-of-the-knight-2 ruby 215, CG 100%, rank 1, 2026-10-25):

- boolean xor is `!=`: `q<t!=$_>?W` = `(q<t) != ($_>"W")`. `^` has lower precedence than `<` and sits with `|`, so `$_>?W^q<t` parses as `$_ > (?W ^ q)` and is wrong
- a `?X` char literal directly before the ternary `?` needs no space: `$_>?W?l=...:h=...` parses (put the response test last in the condition). After an identifier (`...q<t?`) the space is mandatory
- `Integer#[]` sign bit as 0/1: `q[63]` is 1 for negative q, 0 for any small nonnegative q; `t+s-s*q[63]` = `t` when q<0, `t+s` otherwise (edge-rule selector, no ternary)
- low-bit set in one expression: `x|1^t&1` returns x with low bit `(t+1)&1` (`&` binds above `|`/`^`; `|` and `^` are left-assoc); it is a no-op when the bit is already right
- reuse a dead array slot instead of a new variable: after the x axis, store its bound in the size array (`z[i]=l`, final `puts z*" "`), leave the position array `a` alone so the y phase's jumps print the last jumped x (a pure 1D y comparison) and the last jump `(lx,ly)` is printed from z
- `(0..3).map` is 1 B under `4.times.map`
- `$>.sync=1` still mandatory (bare `puts` on a pipe buffers, referee times out)
