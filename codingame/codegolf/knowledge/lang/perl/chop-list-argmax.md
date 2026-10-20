# chop on a sorted list = argmax index

`print chop(@a),$/while$|=@a=sort map<>.$_,0..7`: keys `"h\ni"` (line kept with its newline, then index) sort as strings; `chop(@a)` returns the last char removed = index of last element = last max. `$|=@a=...` is truthy (8) and flushes; `$/` as newline is 1 byte under `"\n"`. 46 bytes, 15/15 local on the-descent. Verified locally 2026-10-20.
Dead ends: `$/=\16` chunk + `index($_,(sort/./g)[-1])/2` 53; `@a[map<>,0..7]=0..7` needs reset.
