# `0+$_` = first field of the line, 1 byte under `$F[0]`

With `-a`, `$F[0]` (5 bytes) as a hash key or number can be `0+$_` (4): a line `"3 5 LEFT\n"` numifies to its leading integer 3 (no `-w`, no warning shown), `"-1 -1 NONE"` to -1. As a hash key `$e{0+$_}` stringifies back to `"3"`, so it finds keys stored from `@F` elements. `$e{+$_}` does not numify (key is the whole line, verified).

Verified: dont-panic ocaml 90 -> 89, CG 100% (2026-10-22); output identical to the `$F[0]` core on a 3000-line random input. Applies to every perl core that keys on the first field (dont-panic perl-wrapper cells in C, Python, PHP, Lua, Haskell, Clojure, Scala, Ruby, perl native).
