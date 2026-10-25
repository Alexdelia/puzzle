# slurp + regex columns

`#!perl -0p` slurps stdin into `$_` and prints `$_` at the end: set `$_=answer` last, no `print`. `@y=sort{$a-$b}/ (.+)/g` takes the 2nd column, `/(.+) /g` the 1st, header line skipped automatically (no space). `sort{$a-$b}` beats `$a<=>$b` by 2 bytes when every pair diff fits 32-bit signed (< 2^31, see [sort-comparator-32bit](sort-comparator-32bit.md)). Median abs-dev: `$t+=abs$_-$y[@y/2]for@y`. Tested locally (system perl, network-cabling 9/9), not on CG referee.
