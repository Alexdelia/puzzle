# Haskell: plan-once DFS via concat comprehension

the-bridge-episode-2: `f x s y p=concat[if win then[q]else f e n z q|j<-[-2..2],let{...},length z>=read v]`, `head$f ...`; command index j with `quot j 2` (speed delta, -j/2), `j*mod j 2` (lane delta: UP -1, DOWN +1), `words"A B C"!!(j+2)`. Slices `take(e-g)(drop g row)`. 398 B, CG 100%, 23/23.
Trap: `interact` with `lines` waits for EOF on interactive stdin if a list comprehension runs to the end of the lines: bound it with `take(read m)`.
