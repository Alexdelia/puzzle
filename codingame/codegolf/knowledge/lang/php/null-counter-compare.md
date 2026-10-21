# Uninitialised loop counter compares as bool

`$i<$x-$a` with `$i` null compares as bool (null<-31 is true): wrong. Start the counter with `++$i<=...` in the first test of the loop body (`for(;;)`), later tests use `$i<=`. Saves 5 bytes over `for($i=0;;$i++)`. Verified power-of-thor 2026-10-21 (local referee 8/8).
