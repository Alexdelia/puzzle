# `-F\D\K` splits after every non-digit: words on the init line, chars on direction lines

Verified 2026-10-25 locally (perl 5.42; the same syntax is plain 5.34).

`perl -F'\D\K' -aE'...'`: `\D\K` matches a non-digit with `\K` resetting the match start,
so `-a` splits *after* each non-digit and keeps the non-digit in the preceding field:

- `"4 8\n"` -> `("4 ", "8\n")` (no `-l`) or `"4 8"` -> `("4 ", "8")` (with `-l`)
- `"UR\n"` -> `("U", "R")`; `"U\n"` -> `("U",)`

So one split mode serves both the numeric header line and the 1-char direction lines
(`for@F` replaces `for/\D/g`, 3 B under), provided line 1 is harmless under `$$_`:

- `${"4 "}`, `${"8\n"}`, `${""}`, `${"4x"}` are writable symbolic-ref names; `$4` is NOT
  (`Modification of a read-only value attempted`).
- With `-l` the newline is chomped, the last field is the bare `"8"` and `$$_` dies on line 1.
  Without `-l` the newline stays in the field, so `-aE` (which implies `-n`) is safe.
- The header still costs `($L,$U)=@F` (same as the default word split) because W numifies
  from `"4 "`; only the apply gets shorter.

With the same header/mids/print shape as the `-alp` body, replacing `for/\D/g` by `for@F`
gives a 76 B body (3 under) and works when run as `perl -F'\D\K' -aE 'BODY'`; verified 16/16
on the local referee (bash launcher, 2026-10-25). `say` needs `-E` (no `-l`), which is fine in
an exec wrapper but fatal on a Perl cell shebang. It is NOT a win in a Ruby wrapper: the shell eats backslashes in an unquoted
`-F\D\K`, so the Ruby source must quote or double them, which costs more than the 3 B saved
(Ruby total 103-105 vs 98 for the plain `-aE` route). It can win in hosts whose wrapper does
not add quote/backslash cost, or in a Perl cell if the newline problem is solved cheaper than
`-l` (+2 for `\n` in the print loses the 3 B gain).

Trap: the line-1 digit fields are only safe because of the trailing newline; do not add `-l`.
