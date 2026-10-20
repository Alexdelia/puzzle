# Walrus inside a slice bound

`r[(x:=w*k)-w:x]` evaluates the start first, so `x` is bound for the stop: 4 bytes cheaper than `for x in[w*k]` inside a comprehension (verified ascii-art 113 -> 107, local 13/13 + CG).
