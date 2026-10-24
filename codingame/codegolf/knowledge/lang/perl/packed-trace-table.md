# Packed action table for a fixed scenario

For an interactive puzzle whose hidden scenarios are known exactly (see `knowledge/lang/_shared/trace-hardcode.md`), the whole program can be a packed table plus a printer:

- actions are one byte per item: a cell `x + B*y` with `B > max x` (fall-2 hidden: `x+14y` in 15..115). Chosen so no byte is NUL, > 127, `'` or `\`: a single-quoted literal needs no escape (control bytes pass the CG submit, NUL does not).
- select the scenario's string with `(split/~/,'<fields>')[$expr]`; `$expr` from the init first line after `$|=<>=~/ /` (`$` = W, `$'` = H). A key with 7 values needs 7 separators even when empty; `(W+7*H)%8` was the cheapest distinct key for the fall-2 hidden.
- decode and print in one `map`: `map{printf"%d %d %sT\n",$_%14,$_/14,$i++%6?RIGH:LEF}unpack"C*",(split/~/,'<fields>')[$expr]`
  - `unpack"C*"` turns the literal into byte values; `$_%14,$_/14` are x,y
  - the action variant comes from a counter (`$i++%6?...`) when the table is ordered so variants alternate; LEFT/RIGHT share the `T` suffix in the format (`format-shared-suffix.md`)
  - then `print"WAIT\n"x99` (or the game's no-op) once; no input is read after the first line
- a 2-element rotation class ({2,3},{4,5} in the-fall) makes LEFT == RIGHT, so such actions can satisfy either variant for free; only the few forced LEFT actions constrain the ordering.

Verified: the-fall-episode-2 perl 541 -> 187, CG 100% rank 1 (2026-10-24), no shebang, no `use`, source length 187 B with 70 B of table. 63 actions, 7 scenarios.
