# exec perl one-liner

- `exec"perl -anE'...'"` = 16 bytes wrapper + core, 15/15 local on dont-panic (96 total vs native ruby 125). Same core as `knowledge/lang/c/system-perl.md`.
- no `#` in core (interpolation); `exec` shorter than `system`; uses /bin/sh (dash)
- native ruby needs `$>.sync=1` (10 bytes) for interactive output
- unverified on CG
- ruby string pitfalls (verified shadows-of-the-knight-episode-1): in `exec"..."` `\d` loses its backslash (`\\d` needed) and `"` needs `\"`; `exec %q(...)` (space required, `exec%q` parses as modulo) keeps `\d` and `"` raw, same total length when both escapes are needed. shadows-of-the-knight-episode-1 ruby: 111 = `exec %q(perl -E'<perl core>')`, 16/16 local, not CG-verified (submit cap hit)
- the-labyrinth ruby 249, accepted on CG (17/17 local): `exec"perl -anE'...'"`, ruby double quote needs `\\n`/`\\K` doubled (or avoid them: 4-arg `substr($_,$F[0]*($d+1)+$F[1],1,R)eq C` marks Rick and tests C in one, no `\n`/`\K`); `exec%q(...)` is a syntax error (parsed as modulo), `exec %q(..)` costs 2 more than `exec"`
- native ruby port of the same flood: 275 (loses to the wrapper by 26): neighbour probe `[-1,1,-d,d].map{m[i+_1]}.index ?X` replaces the Rick regex and direction letters; state var must be declared outside `loop{}` (block-local otherwise); `$<.first(r)*""` reads r lines; regex with `#{c}` needs `/o` only for speed (not needed)
