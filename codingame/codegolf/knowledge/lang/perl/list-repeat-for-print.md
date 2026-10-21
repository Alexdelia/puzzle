# list repetition schedule with `print for`

Verified on CG (mars-lander-episode-3 perl 54, 2026-10-21).

- `print"$_ 4⏎"for(5)x9,(28)x32,(0)x1e5` prints a run-length schedule: `(LIST)xN` repeats the list, per segment `(r)xL,` = 4 + digits, cheaper than `"r 4⏎"xL,` (7 + digits) once the loop header (`print"$_ 4⏎"for` 15) is paid
- a count can read input inside the list: `(5)x(208-9*<>)`; the whole list is built (and `<>` blocks) before the first print
- negative count gives an empty run: `x(C-K*<>)` can vanish on one input
- big total output (> 64 kB pipe + 8 kB stdio buffer) flushes itself without `$|=1` and keeps the process alive blocked on the pipe (see ../_shared/pipe-block-keepalive.md)
