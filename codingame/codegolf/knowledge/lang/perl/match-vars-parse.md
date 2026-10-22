# Perl: match variables as cheap parsers (verified locally, blunder-episode-1)

- `$w=/ /+$'` = 1 + number after the first space (`$'` evaluated after the match, "5\n###..." numifies to 5): 2 bytes under `$w=/ .+/+$&`
- `$p=$-[!/@/]` = index of first `@` in `$_` (the `!/@/` runs the match, index is 0 for `@-`): 1 byte under `/@/;$p=$-[0]`
- `$k=$-[0]if SENW=~$_` = index of char in a letter set, and `$` as cell char matches empty at end so `$k` becomes 4 = natural stop marker
- PITFALL: a list slice with an out-of-range index yields undef, not an empty list (perl >= 5.22): `push@o,(A,B)[5]` pushes undef
- PITFALL: `exit print$o` is fine but `print$o,exit` exits before printing

## Middle field of a 3-field line: `/ .+ /` (verified on CG, the-fall-episode-3 perl 572 -> 571, 2026-10-22)

On "X Y WORD" (single spaces, no trailing space) `<>=~/ .+ /` sets `$`` = X, `$&` = " Y " (numifies to Y), `$'` = "WORD\n". Greedy `.+` spans to the last space, so with exactly two spaces it is 1 byte under `/ \d+ /`. Breaks if the line can carry a third space.
