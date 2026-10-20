# interactive init reads: pitfalls and short forms

Verified locally (the-bridge-episode-2, 2026-10-20):

- `(<>,<>)` and `f(<>)` are list context: `<>` slurps to EOF, hangs on an interactive stdin. Use `$R[$_]=<>for 0..3` or `scalar<>`; `substr<>,2,1` is scalar already
- `$R[$_]=$_>3?0 x99:<>for 0..4` reads 4 lines and appends a sentinel row; `0 x99` (with the space) is 99 zeros, `0x99` is hex
- `map{...;$1}` returns aliases: a later regex resets `$1`. Use `substr<>,2,1` or `$1+0`
- `A.B!~/0/` tests only B (`=~`/`!~` bind tighter than `.`): write `(A.B)!~/0/`
- `!$i-($i>3)` gives 1/0/-1 steps from an index, 3 bytes under `($i<1)-($i>3)`
- args evaluate left to right: `f(0,scalar<>,'',map{substr<>,2,1}1..$M)` reads the speed line then the bike lines inside the call
- non-recursive-critical temporaries (recomputed each loop pass) need no `my`: only values used after the recursive call do
