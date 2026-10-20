# sum{arr.pop-_1} on sorted array

`y.sort!.sum{y.pop-_1}` = sum over i<n/2 of Y[n-1-i]-Y[i] (block pops tail while iterating head, stops mid; odd middle gives 0). Verified network-cabling 21/21 local, 15 bytes shorter than `sum{(_1-y[y.size/2]).abs}`. Also `gets;` is 3 bytes shorter than `.drop(1)` before `$<.map`.
