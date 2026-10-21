# Scanf: whitespace and mixing with read_int

Verified locally (OCaml 4.14.2, `agent/tool/lang/ocaml/build.sh`) and on CG (dwarfs row 127, 2026-10-21).

- `%d` does NOT skip leading whitespace: `Scanf.scanf"%_d%d"` raises `Scan_failure` on the `\n` after line 1. Put a space before each `%d` (`" %d %d"`), a format space eats any run of spaces/newlines
- `read_int()` (or `read_line()`) first, then `Scanf.scanf`: works, `Scanf.Scanning.stdin` reads through the `Stdlib.stdin` buffer, so Scanf continues right after the consumed line. Saves the `Scanf.scanf"%d"(fun n->...)` wrapper: `List.init(read_int())(fun _->Scanf.scanf" %d %d"(fun a b->b,a))` reads `N` then `N` pairs
- the reverse order (Scanf first, then `read_line`) is likely unsafe (Scanf buffers ahead), unverified
- `List.(...)` local open around the whole program (`List.(let e=... in print_int(...))`) is cheaper than `open List;;` + `;;`
