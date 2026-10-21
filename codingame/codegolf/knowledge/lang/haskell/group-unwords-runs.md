# group + unwords for run-length output (verified, chuck-norris 131 bytes)

`import Data.List` then `unwords[header++('0'<$a)|a@(x:_)<-group bits]`: unwords adds separators so no `tail`, `'0'<$a` replaces each element by '0', `drop x"00 "` picks header from Int bit x. Beats hand-written span recursion by 11-14 bytes.
Bits: `fromEnum c`div`2^i`mod`2|c<-s,c>'\n',i<-[6,5..0]` (div/mod infixl 7, ^ binds tighter: no parens). `[64,32..1]` is a trap: enumFromThenTo stops at 32.
`else' '` lexes as one identifier; write `else ' '`. `interact$ \s->` needs the space (`$\` is an operator).
