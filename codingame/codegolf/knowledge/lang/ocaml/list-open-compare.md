# `compare` inside `List.(...)`

Verified locally 2026-10-22 (OCaml 4.14.2): inside `List.(...)` local open, `compare` is `List.compare` (since 4.12, type `('a->'a->int)->'a list->'a list->int`), so `sort compare l` is a type error. On ints use `sort(-)` (2 B shorter than `Stdlib.compare` workarounds and shorter than `compare` itself); safe while differences fit 63-bit `int`.
