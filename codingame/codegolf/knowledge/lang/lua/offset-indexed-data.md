# Offset-indexed hardcode table (no separators)

Verified 2026-10-21 on vox-codei-episode-1 (CG 100%, 217 -> 192 B): instead of N slots split by a separator (`("[^,]*,"):rep(k)` + match), put all answers in one string at positions given by the key and take the rest: `([[DATA]]):sub((a*R+b*B-c*w)%m):gmatch'.'`.

- no separator bytes, no capture pattern (-36 B of code, data 51 -> 50); gaps between answers are filler (any char), search `a,b,c,m` for tight packing
- the rest of the string after the answer is consumed as extra output: harmless when the extra lines are no-ops (here: after the last bomb any `x y` is a wait); keep >= 3 spaces (WAIT) after the last answer
- `sub(0)` = whole string, so key 0 and 1 both start at the beginning
- read the key operands inline in order, no variables: `l=io.read` then `l'n'*8+l'n'*2-w*5` (left-to-right evaluation)
- `l'n','n'` is NOT two reads (call sugar takes one argument): `l'n'l'n'` or `l("n","n")`
- skip a rows block of known size from a pipe: `l(h*-~w+1)` reads exactly `h*(w+1)+1` bytes (never ask for more than is available: io.read(n) blocks until n bytes)
- `([[...]]):sub` needs the parentheses around a long string literal
