# Counter-letter interact loop (power-of-thor, verified 8/8 local, CG 100%)

`main=interact$(\(a:b:c:d:_)->unlines[['S'|i<b-d]++['N'|i<d-b]++['W'|i<c-a]++['E'|i<a-c]|i<-[0..]]).map read.words` (113)

- lazy `interact` with irrefutable-length pattern `a:b:c:d:_` (not `[a,b,c,d]`, which blocks on EOF), infinite output, energy lines ignored
- `['S'|cond]` as an optional letter, concatenated
- dropping the E term (100 bytes) scored only 50% on CG: no-E assumption in puzzle hidden facts is wrong
