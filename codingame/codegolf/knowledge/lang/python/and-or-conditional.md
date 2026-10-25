# `X and A or B` beats `A if X else B` when A is truthy

`A if X else B` (with no spaces: `Aif Xelse B` is a syntax error) costs
`len(A)+len(B)+len(X)+7`; `X and A or B` costs `len(A)+len(B)+len(X)+6` because `and`/`or`
are 3+2 bytes and need only spaces around `X` (`Xand`/`Xor` merge digits+keywords:
`i&2and'L'or'R'` parses, with only a SyntaxWarning on Python 3.12 for the digit merge).
Verified the-fall-episode-3 python 2026-10-25: `i&2and'LEFT'or'RIGHT'` (21) replaced
`'LEFT'if i&2else'RIGHT'` (23) in a CG-100% 134-byte plan hardcode (-2 B).
Only valid when A is truthy for every X that selects it (a non-empty string, a non-zero
number); otherwise `or` falls through to B.
