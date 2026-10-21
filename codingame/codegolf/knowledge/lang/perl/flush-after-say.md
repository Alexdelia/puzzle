# `$|=cond&&say...`: set autoflush after the print

Verified 2026-10-21 (dont-panic, local referee 15/15 lua and scala, scala CG accepted 135): setting `$|` to true flushes STDOUT right away (perlvar), so `$|=COND&&say+EXPR` works interactively although `$|` becomes 1 only after `say` returned. Lines where COND is false set `$|` to "" but print nothing.

- vs `say+EXPR if$|=COND`: 1 byte shorter (no ` if`) and no space in the code, so a perl core passed as one argv word without a shell (Scala `sys.process.Process("perl -anE<code>")`) needs no quotes at all
