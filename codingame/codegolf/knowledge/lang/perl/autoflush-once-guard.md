# `$|||=` as a run-once header guard

In a `-p`/`-n` turn loop, `$|||=($a,$b,$x,$y)=@F;` assigns the header only on iteration 1: `$|` starts at 0, the list assignment in scalar context returns 4, `$|` becomes 1 (autoflush on), and every later `||=` short-circuits, so energy lines never overwrite the vars. 4 bytes under `$|=($a,$b,$x,$y)=@F if$#F;` (no field-count test, no `if`). Tokenizes as `$|` `||=`, no space needed.

Needs the RHS truthy on the header (any non-empty list) and no other write to `$|`.

Verified: power-of-thor perl 90 (`#!perl -alp`), local 8/8, CG-accepted 2026-10-22 (submit.sh, rank #29).

Comma list with extra reads (multi-line header): parenthesise the whole RHS, else the
commas sit outside `||=` (deparse `$| ||= (...), readline, readline, ...`) and the reads
run every iteration (deadlock in interactive puzzles). The RHS value is assigned to `$|`,
so the last operand must be numeric/truthy: put the list assignment last
`$|||=((<>,<>,<>),($L,$U)=@F)` or `$|||=(<>,<>,<>,($L,$U)=@F)`; a trailing `<>` line
(e.g. a direction "UR") numifies to 0 and silently disables autoflush (turn-1 timeout).
Verified: shadows-of-the-knight-episode-1 perl 91 (CG 100%, `#!perl -alp`, 3 init lines
read on iteration 1), local 16/16 + 13/13 hidden, 2026-10-24. `$|++or RHS` also
short-circuits and ignores the RHS value, but costs 6 vs 5 (`$|||=`) before the RHS.
