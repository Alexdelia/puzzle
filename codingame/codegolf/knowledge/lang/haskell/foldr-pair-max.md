# foldr max over (key,value) pairs

Verified local referee 17/17, temperature 72 B.

- `foldr(max.(\t->(-t*t,t)).read)(-9^9,0).tail.words` replaces `maximum.(seed:).map f`: foldr with a seed pair saves 4-6 B and also covers the empty list
- max with key `-t*t` makes ties go to the larger t (positive) without a tie term
- tuple + `snd` beats list + `last` by 1 B
- Integer defaulting works for `read` with no annotation
