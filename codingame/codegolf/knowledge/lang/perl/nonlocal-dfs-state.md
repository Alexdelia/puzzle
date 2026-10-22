# Drop `local` in a recursive DFS when nothing reads the state after the child call

In `sub f{...for$c(LIST){...;f(ARGS)&&return$c}}`, LIST and ARGS are evaluated before each child call; if nothing in the loop body reads the topic `$_` or a scratch hash after `f(...)` returns, both can be plain globals (the child clobbering them is harmless). Reset both in one statement: `%h=($_)=@_;` (list assignment returns its lvalues, so `%h=($_)`: one stray key = the state, value undef, harmless for counters). 8 under `local($_,@r,%h)=@_;`.

- needs every per-iteration use of `%h` moved before the loop: e.g. kill collided movers once at entry, `$_*=$h{$_>>2}<2for@_` (dead -> 0, modifies the arg temporaries in place), then the child call is a plain `f(map S,@_)`
- the topic keeps the caller's (pre-kill) value while `$_[0]` holds the killed one: `!$_[0]|$M{"@_@z$."}++&&return$_>>2==$X` tests "mover dead or memo hit" and still answers "reached the exit" from the copy

Verified: the-fall-episode-3 perl 584 -> 572, local 11/11, CG 100% (2026-10-22). See [[local-topic-recursion]], [[foreach-arg-alias]].
