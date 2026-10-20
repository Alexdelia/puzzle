# exec perl one-liner

- `exec"perl -anE'...'"` = 16 bytes wrapper + core, 15/15 local on dont-panic (96 total vs native ruby 125). Same core as `knowledge/lang/c/system-perl.md`.
- no `#` in core (interpolation); `exec` shorter than `system`; uses /bin/sh (dash)
- native ruby needs `$>.sync=1` (10 bytes) for interactive output
- unverified on CG
