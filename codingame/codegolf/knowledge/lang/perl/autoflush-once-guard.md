# `$|||=` as a run-once header guard

In a `-p`/`-n` turn loop, `$|||=($a,$b,$x,$y)=@F;` assigns the header only on iteration 1: `$|` starts at 0, the list assignment in scalar context returns 4, `$|` becomes 1 (autoflush on), and every later `||=` short-circuits, so energy lines never overwrite the vars. 4 bytes under `$|=($a,$b,$x,$y)=@F if$#F;` (no field-count test, no `if`). Tokenizes as `$|` `||=`, no space needed.

Needs the RHS truthy on the header (any non-empty list) and no other write to `$|`.

Verified: power-of-thor perl 90 (`#!perl -alp`), local 8/8, CG-accepted 2026-10-22 (submit.sh, rank #29).
