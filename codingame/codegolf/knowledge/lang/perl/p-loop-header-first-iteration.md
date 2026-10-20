# `-alp` loop as interactive turn loop, header on iteration 1

Verified on CG power-of-thor (100%, 96 bytes).

- `#!perl -alp`: each stdin line is one iteration, `$_` printed at the end with `-l` newline. Header line is iteration 1, energy lines the rest
- init only on the header: `($a,$b,$x,$y)=@F if$#F` (energy lines have 1 field). The header iteration also prints a move: CG accepts the move one line ahead of the energy line
- autoflush for free: `$|=($a,$b,$x,$y)=@F if$#F` (list assignment in scalar context = 4, truthy, `$|` becomes 1) saves 2 bytes vs `$|=1;`
- always set `$_` in the body, or `-p` prints the raw input line
- negative index on a 3-list: `("",S,N)[$v]` for `$v` in -1,0,1 (barewords, no strict)
