# Local helper for repeated conditional letters

Verified on CG (power-of-thor row 146, 2026-10-22).

- several `(if cond then"X"else ...)` with the same shape: bind once inside the loop, `let r l p q=if i+q<p then l else""in` then `r"S"b d^r"N"d b^r"W"c a^r"E"a c` (curried calls need no parens, no space before a string literal). 66 B vs 85 for two nested if-chains
- `for i=0 to 99999 do ...done` is 1 B shorter than `let rec f i=...;f(i+1)in f 0`; for interactive open-loop output 1e5 flushed `print_endline` lines exceed the 64 KB pipe, so the process blocks instead of exiting early (knowledge/lang/_shared/pipe-block-keepalive.md)
- int literal followed by a letter (`0to`, `99do`) is a lexer error (literal modifier), spaces stay
