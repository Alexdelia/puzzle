# interactive golf idioms (verified on shadows-of-the-knight-episode-2, local)

- `z,_,a=4.times.map{gets.split.map &:to_i}` reads 3 init lines + UNKNOWN line
- response compare: `gets;/S/?x:y` (bare regex matches `$_`), `$_>?W` true only for "WARMER" (longer string > "W")
- `q>t==$_>?W` parses as `(q>t)==($_>?W)`
- `c=t+a[i]=q` assigns and sums in one go
- `h=c-1>>1` instead of `(c-1)/2`
- `$>.sync=1` mandatory: without it the referee times out on turn 1
