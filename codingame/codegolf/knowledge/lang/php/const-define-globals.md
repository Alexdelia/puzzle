# const/define instead of `global`

Verified the-fall-episode-3 (php 1267 -> 1192, 2026-10-21): in a recursive solver with helper functions, read-only data goes in `const E="...",T=[...];` (const arrays indexable, `E[$i]` on const string) and runtime values in `define(v,$w+1)` (bare name as string, warning only). Saves every `global$x;` and `$`. Only the mutated grid keeps `global$g`.

- `printf("%d %d %s\n",...,$l?LEFT:RIGHT)` with bare constants and a literal newline inside the string (1 byte vs `\n` 2)
- a helper `function R(){fscanf(STDIN,"%d%d%s",$x,$y,$s);return ...}` reads repeated lines
- `+$x` on `explode` pieces tolerates the trailing "\n" (notice only)
