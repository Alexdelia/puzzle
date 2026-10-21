# Silence notices on heavy solvers (the-fall-episode-2, verified on CG)

- a PHP port that leans on undefined offsets/constants (8 MB of notices on stderr locally) scored 84% on CG (same code, 100% locally); adding `error_reporting(0);` made it 100%. Cause reproduced locally 2026-10-21 (referee validators 17-19, big maps 13x15..15x19): ~400k notice lines per game (undefined constants RIGHT/LEFT in a `foreach` literal, undefined offsets), PHP 3-4x slower than the python port, timeout on turn 2 at 150 ms; same code with `display_errors=0` passes 26/26
- cost 19 bytes; `<?error_reporting(0);` with short open tag saves 3
- rule: when a PHP golf generates notices in a hot loop (DFS, recursion), silence them
- `const T='..',P=[..];` at top level gives string/array constants readable in functions without `global`; `$a=T[$i]` and `P[$k]` index them directly
