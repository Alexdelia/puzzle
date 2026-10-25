# Read the first k lines with star-unpack

`a,b,*c=open(0)` reads the whole stdin and unpacks it: `a`,`b` = first two lines (newline kept),
`c` = list of the rest. 14 B, +1 for the separator -> the cheapest way to get Python's first two
lines; `a,b=open(0)` raises `too many values to unpack` (the iterator has every line, not two).
`a,b=input(),input()` is 19 B and strips the newlines; `a=input(),input()` is 17 B but gives a tuple
(`a[0]`,`a[1]` cost more in the key). Verified blunder-episode-3 2026-10-25: native 101 B hardcode,
CG 100%.
