# abs in sort comparator

`abs$a-abs$b` parses as `abs($a-abs$b)` (named unary op binds looser than `-`): wrong result. Use `$a*$a-$b*$b||$b-$a` (verified on temperature). Perl 50 B: `<>;print+(sort{$a*$a-$b*$b||$b-$a}split$",<>)[0]+0` (`+0` gives 0 on empty list).
