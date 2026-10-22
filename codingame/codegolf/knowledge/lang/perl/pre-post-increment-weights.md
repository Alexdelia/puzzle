# centered index weights from ++$s / $s++

Two sums with the same centered weight `2i-N+1` (i = 0..N-1): `$s=-<>;...$b+=++$s*X,$a+=$s++*Y for<>` reads N, pre-increments for the first sum and post-increments for the second, so both see `1-N, 3-N, ..., N-1` (step 2, sum exactly 0). 2 under `$s=1-<>;...$a+=$s*X,$b+=$s*Y,$s+=2`.

Trap: `$'for` parses as `$main::for`; end the statement on a bare `log` + ` for` (space kept) instead.

Verified: blunder-episode-3 perl 121 -> (with glob table) 110, local 26/26, CG-accepted 2026-10-22.
