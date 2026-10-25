# print two ints with a space, coordinates without an f-string for both

`print(s+f'{x}',y)` prints `s+"x y"`: the default `sep=' '` inserts the space
between the two arguments, so only the first coordinate needs a string.
`print(s+f'{x} {y}')` is 2 bytes longer; `str(x)` is 1 longer than `f'{x}'`
in `f'{x}'`'s 6 extra chars? (measured: `f'{a&15}'` 9, `str(a&15)` 9, equal).
Verified vox-codei-episode-2 python 171 CG 100%: `print('WAIT\n'*(a*b%23)+f'{a&15}',b&15)`.

Trap: a literal newline cannot appear inside a single-quoted Python string
(unlike Ruby); `'WAIT\n'` must be the 2-byte escape.

Trap: `bytes.find(str)` raises TypeError; making the haystack bytes needs
`input().encode()` (+9) or a str data literal and `ord` on every char (+20),
so byte-marker lookups lose to `str.split()` + integer key.
