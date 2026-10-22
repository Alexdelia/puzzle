# Printf `%.*s` is width, not precision

Verified locally (OCaml 4.14.2, `$CG_OCAML`, 2026-10-22).

- `Printf.printf"[%.*s]"5"W"` prints `[    W]`, `(-3)"S"` prints `[S  ]` (left-justified), `0"N"` prints `[N]`: the `*` after `.` acts as a width for `%s`
- so no C-style `%.*s` truncation: a string cannot be gated to "" by a 0 precision; use an `if` helper instead
