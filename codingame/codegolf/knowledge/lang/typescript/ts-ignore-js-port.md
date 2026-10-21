# //@ts-ignore line: port JS golf as is

Verified 2026-10-21 (music-scores, CG 100%, 233 B, rank #1): `//@ts-ignore\n` (13 B) before a one-line program suppresses all type errors of that line (undeclared globals, `print`, `boolean|0`) while tsc still emits. `print` exists at runtime (JS polyfill). So TS cost = JS cost + 13, cheaper than `let` lists (`let R:any=readline,...,x,q,o=[],i` was 249 vs 240). The whole program must sit on the single line after the comment.
