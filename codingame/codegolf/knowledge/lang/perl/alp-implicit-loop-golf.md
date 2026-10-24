# `-alp` per-line loop: shebang switches, 5-byte first-line gate, blank suppression

Verified locally 2026-10-24 (shadows-of-the-knight-episode-1 perl, 16/16 local,
92 B `-alp` variant equal to the native 92 B; not submitted because no gain):

- `perl` honors switches on the `#!` line even when invoked explicitly as
  `perl main.pl` (CG launcher does this): `#!perl -alp` gives autosplit `@F`,
  chomp + `$\="\n"`, implicit `while(<>){...;print}`. Code after the switches on
  the shebang line is NOT executed; a `;` there is parsed as a switch
  (`Unrecognized switch: -;`).
- Cheapest first-iteration gate in a `-p`/`-a` body: `if!$|` (5 B). `$|` is 0 on
  line 1; any `$|=...` in the header (e.g. `$|=($L,$U)=@F`) makes it truthy for
  every later line. `if!$L` also works (5 B, W is never 0).
- Blank-line suppression: with `-p` and NO `-l`, `$_=""` prints 0 bytes (nothing
  reaches the referee). With `-l`, `$_=""` prints one empty line. Use the no-`-l`
  form to silence init lines.
- `$$_=EXPR` on a line that contains digits dies "Modification of a read-only
  value attempted": `${"4"}` = capture var `$4`, which is read-only while unset.
  Use `/\D/g` instead of `/./g` (+1 B) to run a per-line symbolic assignment
  ungated on a numeric first line; a space maps to `${" "}`, which is writable.
- Cost arithmetic for a per-turn interactive cell: `-alp` = 11 B shebang + 1 B
  newline + `if!$|` gate (5) replaces the native `{...;redo}` wrapper (7) and the
  explicit read (`for<>=~/./g` 11) and the print (`print"@m\n"` 10 -> `$_="@m"` 7):
  net ~0. It wins only when the header is `@F`-shaped and the body is long.
- `glob` is a greedy list operator: `glob<>.<>,<>,<>` fails "Too many arguments
  for glob"; the multi-line read needs the dots (`glob<>.<>.<>.<>` 15 B) or
  `glob(<>.<>)` (+2). Comma-separated `<>` reads after a parenthesized glob do
  work as list elements: `($L,$U)=glob(<>.<>),<>,<>` reads 4 lines with L,U from
  the first two words (28 B with `$|=`, 1 B longer than the dotted form).
