# Lua golf tricks (verified, music-scores 2026-10-21)

- `w,h,s=io.read("n","n","a")` reads two numbers then the rest of stdin in one call
- `s:gsub(pat,s.rep)`: string methods are reachable via any string (`s.rep`, `s.char`, `s.byte`), no `string.` prefix; gsub passes captures as arguments, so `s:gsub("(.) (%d+) ?",s.rep)` expands RLE `B 5 W 3` without a callback
- `s.char(a,b)` emits several chars in one call
- leading newline left in `s` after `io.read("n","n","a")` shifts 1-based indices by 1 more: pixel f is byte f+2
- two-valued byte to char: `81-b%2*9` maps B(66)->81, W(87)->72; brute-force `K-b%n*m` forms instead of `*3//7`
- stepping a column in a flat string: `i=x while i<M and cond do i=i+w end`, row = `i//w`
- crash at end is free on io puzzles, but nil compare mid-run is not: bound loops explicitly
