# local push of one element

`local$q[@q]=X` appends X to `@q` for the current scope; on scope exit the element is deleted and `@q` shrinks back to its old length (perl 5.34, verified: recursion printing `scalar@q` returns to 0 after the walk). 2 B under `local@q=(@q,X)`. Works inside a `map`/`for` body restored per iteration, so a DFS path stack needs no `push`/`pop`.

Same session: reading header lines inside a fixed-count loop list, `$R.=pack A99,~~<>for$M=<>,$V=<>,1,2;` (list is evaluated before the body, 4 iterations, the 2 headers are read first): 1 B under `$M=<>;$V=<>;...for 1..4`.

Verified on CG 2026-10-22: the-bridge-episode-2 perl 267 (100%).
