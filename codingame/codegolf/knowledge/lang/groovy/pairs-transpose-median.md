# Groovy: parse "N then N pairs" and median deviation

Verified local 21/21 on network-cabling (130 bytes, not CG-submitted, 2026-10-21):

- `(x,y)=System.in.text.split()*.toLong().tail().collate(2).transpose()`: header dropped by `tail()`, columns into x and y
- `y.sort().sum{(it-y[y.size()>>1]).abs()}`: sort returns the list, so y is already sorted inside the closure
- `String[] as int[]` yields char codes, not parsed numbers
