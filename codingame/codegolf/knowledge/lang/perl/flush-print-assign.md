# Assign print's result to $| (flush + print in one operator)

`$|=print EXPR," 4\n"while<>` replaces `$|=1;print EXPR," 4\n"while<>`:
`$|=` takes `print`'s return value (1, true), and assigning 1 to `$|`
flushes the handle immediately, after the same print. Saves 2 bytes.

Verified on CG-style referee 2026-10-24 (mars-lander-episode-3 perl cell
`$|=print int($_/100-30)," 4\n"while<>`, 36 B): the first turn's answer was
flushed on the iteration that printed it (game reached turn 92, no
"timeout turn 1"). The RHS print runs before the assignment, so the line
printed in that iteration is flushed by the assignment that follows it.

`COND&&$|=print...` is a syntax error (`Can't modify logical and (&&) in
scalar assignment`); put `$|=1;` separately before the conditional form.
