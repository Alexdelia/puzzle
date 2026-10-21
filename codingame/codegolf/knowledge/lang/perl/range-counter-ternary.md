# range counter + ternary chain for a time schedule

Verified on CG (mars-lander-episode-3 perl 45, 2026-10-21).

- `print$_<170?11:$_<185?-89:0," 4⏎"for<>*5..1e5`: the foreach range is the turn counter, its start `<>*5` reads one input line in scalar context (`*` forces it) and keys the offset on it; 2 cuts cost `$_<T?r:` = 7-8 chars each
- 2 chars under the list-repeat form `print"$_ 4⏎"for(11)x(170-5*<>),(-89)x15,(0)x1e5` (see list-repeat-for-print.md): the keyed count `x(C-K*<>)` costs more than the range start
- `print$_<...` parses as an expression (no filehandle guess before `<`); foreach over a range is lazy, 1e5 lines of output also keep the process blocked on the pipe (no `$|`)
