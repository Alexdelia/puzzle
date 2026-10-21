# Clojure: fill a buffer and keep it without `let`

`(doto(byte-array 819)(#(.read System/in %)))` returns the filled array: `doto` threads the object as FIRST arg, so `(doto b(.read System/in))` calls `b.read(System.in)` (IllegalArgumentException); wrapping in `(#(...))` puts it where you want. Saves ~11 B vs `(let[b(byte-array 819)](.read System/in b)...b)`. `(hash(vec b))` then gives a 32-bit key over the first chunk (init + first turn line on CG interactive). Verified CG 2026-10-21 (vox-codei-episode-1 213).
