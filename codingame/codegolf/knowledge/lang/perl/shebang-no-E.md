# `-E` on the shebang line fails

`#!perl -nlE` dies: `Can't emulate -E on #! line` (perl 5.42 locally, verified). Use `-nl` and `print`, or `say` needs `use feature`. `print+(LIST)[...]` under `-l` prints the list joined by `$,` (undef) plus newline, so a list-slice row needs no `join`.
