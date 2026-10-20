# Flat padded grid: no bounds checks

Verified vox-codei-episode-2 (CG 982 bytes).

- read rows as `G=''.join(input()+'#'for _ in[0]*H)+'#'*S` with `S=W+1`: one `#` column on the right and one `#` row at the bottom
- cell = `y*S+x`, neighbours `c+d` for `d` in `0,1,-1,S,-S`; left of col 0 wraps to the previous row's `#`, above row 0 wraps (negative index) to the bottom `#` row, so `G[c+d]=='#'` is the only test (valid when the walk stops at the first wall)
- decode: `c%S`, `c//S`
- store answers as ready strings: `P[w]="%d %d"%(c%S,c//S)` then `print(P.get(t,"WAIT"))`
- transpose paths per node into positions per time: `T=[*map(set,zip(*N))]`
- `'#'<x` is a one-byte-shorter test than `'#'!=x` when the other chars are `.` and `@`
