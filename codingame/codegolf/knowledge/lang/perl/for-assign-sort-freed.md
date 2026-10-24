# foreach over a list assignment, popping it, dies

Verified 2026-10-24 with CG Perl 5.34 (perl 5.42 too), network-cabling scratch.

`for@y=sort{$a-$b}@y` iterates the array @y itself (a list assignment in the foreach list returns the array's lvalues, not a copy). If the loop body pops/shifts @y, iteration dies:

    @y=(3,1,2); $s-=$_-pop@y for@y=sort{$a-$b}@y
    Use of freed value in iteration at -e line 1.

So inlining the sort into the pair-end loop (`$s-=$_-pop@y for@y=sort...`, would save 2-3 B over `@y=sort...;$s-=... for@y`) is impossible; keep the separate assignment. Same for `shift`. (Iterating a plain pre-assigned `@y` while popping it is fine and is the usual trick.)

Trap: `pop(@x=sort{$a-$b}/.+ /g)` / `shift(@x=...)` are compile errors in 5.34+: `Experimental pop on scalar is now forbidden`.
