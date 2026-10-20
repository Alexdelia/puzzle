# Perl regex flood BFS in a bash wrapper (the-labyrinth, 256, CG #3)

Verified 17/17 local, accepted on CG. `perl -anE` one-liner, interactive.

- backward flood with ONE plain mark `X`: sources `X`, one pass = `s/[.T](?=X|.{C}X)|(?:(?<=X)|(?<=X.{C}))[.T]/X/g` (a single s///g reads the original string, so one pass = one BFS layer, no lowercase/uppercase layering needed). Row stride is C+1 with `\n` mapped to `#` by `y` (no `/s`)
- NO direction letters: mark Rick's cell `R`, loop until a marked neighbour exists, `until/(X)R|R(X)|(X).{C}R|R.{C}(X)/`, then `$#-` (index of the group that matched) picks the word: `qw(a LEFT RIGHT UP DOWN)[$#-]`. Any marked neighbour of Rick at first contact is on a shortest path (neighbour distances are d-1 or d+1). One extra layer would be harmless too (grid is bipartite)
- phase state by tr list: `$s=T if$1eq C` (Rick on C), `eval"y/$s?\n/X#/"`: phase 1 maps `?`->X, phase 3 maps `T`->X and `?`->`#`, `\n`->`#` always. T stays passable in phase 1 (T impassable there hangs: start T can sit in a corridor)
- fallback to C without a branch: `s///g||y/C/X/` (zero substitutions = flood stagnated = no reachable `?`, turn C into a source)
- Rick cell + C test in one: `s/^(?:.*\n){$F[0]}.{$F[1]}\K(.)/R/;$s=T if$1eq C`
- header via `$r||(($r,$d)=@F,$|++,next)` under `-an`; map via `read STDIN,$_,$r*($d+1)` (`@F` strings: `-~$d` is string negation, use `$d+1`); `$|++` needed
- TRAP: CG perl 5.34 variable-length lookbehind `(?<=X|X.{N})` compiles with a warning and matches wrongly (acts like `X.{0,N}`): use two fixed lookbehinds
- TRAP: `for 0,1` aliases `$_` to a constant, `$_=reverse` in it dies; `$_=reverse` double-sweep trick is longer anyway
- hardcoding 15x30 impossible: validators 07 (21x41) and 09 (10x20)
