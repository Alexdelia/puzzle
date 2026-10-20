# Endless loop as subshell recursion

Verified on shadows-of-the-knight-episode-1 (local referee, bash 5.1.16): `f()(BODY;f);f` is 2 bytes shorter than `for((;;)){ BODY;}` and 4 shorter than `f(){ BODY;f;};f`. Variables set in BODY stay visible in the nested calls, so state flows down. Stdin is shared. Each level forks: fine for ~100 iterations at 150 ms/turn.

`f()(((...))||cmd;...)`: `(((` after `f()` parses as subshell + arithmetic, no space needed.

First-iteration trigger: `(($c=${c,},1))||echo X` with `c` unset expands to `((=,1))`, a syntax error, so the `||` branch runs once before any read. A newline read by `read -n1 c` gives empty `c`: same error, same branch.
