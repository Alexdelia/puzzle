# Shared adjacency pattern for grid flood + direction

Verified on CG (the-labyrinth perl 209 -> 192 -> 191, 2026-10-22; 191 -> 190, 2026-10-24).

Grid as one string of rows joined by `\n` (width 30, stride 31). One pattern string tests "this cell has a source `X` as neighbour" and is reused by the flood and by the stop test:

    $f='(?=(?<=X.)|(?<=X\H{31})()|X()|\H{30}X())';
    s/[.T]$f/X/g until/R$f/;
    $_=(LEFT,UP,RIGHT,DOWN)[$#-]

- lookbehinds sit inside the lookahead after the cell, so all 4 neighbours are one `(?=..|..)` group, no `(?:..)` wrapper
- one `s///g` pass = one BFS layer (lookarounds read the pre-substitution string)
- `$#-` of the stop match = index of the last matched `()`: 0 left, 1 up, 2 right, 3 down
- `\H` (not horizontal whitespace) matches `\n` and every grid char, so no `/s` flag; it is one char, so fixed-length lookbehind stays legal in perl 5.34 (CG). Keep the two lookbehinds separate, variable-length lookbehind is wrong on 5.34
- the up lookbehind ends after the consumed cell, so `X\H{30}.` (X, 30 chars between, the cell) is the same assertion as `X\H{31}` (X plus the cell and the 30 in between, fixed length 32): **-1 B**. Same trick applies to any `A\H{n}.` lookbehind where the trailing `.` is the just-consumed char
- step/fixpoint loop: `until/R$f/` keeps the written `X` out of `[.T]`; a fixpoint `1while s/[.T]$f/$#-/ge` also works (no R mark, Rick's cell gets the direction digit) but reading the digit back costs more than `$#-`
- in a shell-quoted wrapper core use `$f=q(...)` (balanced parens) instead of `'...'`
