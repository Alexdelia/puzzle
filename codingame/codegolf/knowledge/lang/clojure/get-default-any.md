# Clojure: `(get v i any?)` as an "everything" default

Verified on the-bridge-episode-2 (CG 100%, 467 B, 2026-10-21).

- grid rows as sets of blocked indices: `(set(keep-indexed #({\0 %}%2)(str l)))` (map literal `{\0 %}` called on the char returns the index iff the char is `\0`, 3 B shorter than `#(if(= %2\0)%)`)
- `(get R z any?)` returns the row set for a valid row, `any?` (true for every index) for a row index out of range, negative included: off-grid rows read as fully blocked without padding rows; `(not-any?(get R z any?)(range a b))` is the cell test, sets and `any?` both used as predicates
- `[M V a b c d S _ & r](repeatedly read)` names the fixed header tokens and leaves the rest lazy, `(take M(take-nth 3 r))` picks one field per 3-token record without reading past it (interactive-safe)
- `(fn f[..]..)` inline named fn + `(some(fn[[c j d b]]..)'[[SPEED 1 0]..])` returns the first non-nil branch of a DFS; quoted symbol tuples print bare with `str`
- a number directly followed by a symbol is a reader error (`0S`), keep the space
