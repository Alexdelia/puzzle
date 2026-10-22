# Flat grid string + `local` recursion state

Verified on CG (the-bridge-episode-2 perl 303, 2026-10-22, 100%).

- grid as one string, rows padded to a fixed stride: `$R.=$_?pack A99,~~<>:0 x99for 1..4,0` (4 input rows padded with spaces, then a sentinel row of `0`). A cell is one offset `y*99+x`; lane change = `+-99`, no separate x/y
- row above row 0 needs no sentinel: the offset goes negative and `substr` counts from the end, landing in the trailing sentinel row (as long as `x < stride`)
- `pack A99,~~<>` pads with spaces; `~~` keeps `<>` scalar (list context hangs on interactive stdin)
- recursion state without `my(...)=@_`: inside the `for` body, `local$s=$s-...` and `local$q=$q.CMD.$/` are restored at the end of each iteration; the sub then reads its bikes from `@_` directly. -8 B vs `my($s,$p,@Y)=@_` + passing them
- TRAP: `@Z=map{..}@_;...f(@Z)` with a global `@Z`: the callee's `@_` aliases `@Z`'s elements without refcount, reassigning `@Z` in the callee frees them and later loop passes read garbage (`perl -we 'sub f{@Z=map$_+1,@_;@Z=(7,8);print"@_"}@Z=(1,2);f(@Z)'`). Can still pass tests by luck. Keep `my@Z`
