# `|0` does not truncate negative numbers

`EXPR|0` truncates a float toward zero only while it is non-negative.
On a negative value Perl's bitwise OR yields an unsigned 64-bit IV:
`sin(1)*-42|0` prints `18446744073709551596` (2^64-20), not `-35`.
Verified 2026-10-24 (mars-lander-episode-3 perl cell: the answer line
`18446744073709551596 4` was refused as an invalid command).

Use `int(EXPR)` (+5 bytes) or `printf"%d ...",EXPR`, which handles
negative floats.
