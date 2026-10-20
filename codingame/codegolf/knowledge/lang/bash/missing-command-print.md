# print only when a variable is set

Verified locally (bash 5.1.16) and on CG (dont-panic submission accepted 2026-10-20).

- `${d:+echo} $out`: when `d` is empty the line runs `$out` as a command, which is not found and writes only to stderr (not judged). Shorter than `[ $d ]&&echo $out` by 2, and side effects inside `$out` (e.g. `${e[f]=$p}`, arithmetic assignments) still happen on every line
- cost: `$out` must not be the name of a real command or builtin (`WAIT`, `BLOCK` are safe)
- arithmetic syntax error inside an expansion aborts the whole top-level command (the enclosing `while` loop included), not just the current iteration
