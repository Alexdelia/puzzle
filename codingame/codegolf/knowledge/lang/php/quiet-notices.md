# Silence notices on heavy solvers (the-fall-episode-2, verified on CG)

- a PHP port that leans on undefined offsets/constants (8 MB of notices on stderr locally) scored 84% on CG (same code, 100% locally); adding `error_reporting(0);` made it 100%. Cause unproven (probably timeouts from notice formatting/stderr writes on bigger hidden maps)
- cost 19 bytes; `<?error_reporting(0);` with short open tag saves 3
- rule: when a PHP golf generates notices in a hot loop (DFS, recursion), silence them
- `const T='..',P=[..];` at top level gives string/array constants readable in functions without `global`; `$a=T[$i]` and `P[$k]` index them directly
