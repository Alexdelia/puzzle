# exec perl one-liner

- `exec"perl -anE'...'"` = 16 bytes wrapper + core, 15/15 local on dont-panic (96 total vs native ruby 125). Same core as `knowledge/lang/c/system-perl.md`.
- no `#` in core (interpolation); `exec` shorter than `system`; uses /bin/sh (dash)
- native ruby needs `$>.sync=1` (10 bytes) for interactive output
- unverified on CG
- ruby string pitfalls (verified shadows-of-the-knight-episode-1): in `exec"..."` `\d` loses its backslash (`\\d` needed) and `"` needs `\"`; `exec %q(...)` (space required, `exec%q` parses as modulo) keeps `\d` and `"` raw, same total length when both escapes are needed. shadows-of-the-knight-episode-1 ruby: 111 = `exec %q(perl -E'<perl core>')`, 16/16 local, not CG-verified (submit cap hit)
