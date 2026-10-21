# Regression golf (blunder-episode-3, verified 241 CG 100%)

- `n=sum(1<$x)` is a Double length (vs `fromIntegral$length x`)
- one helper `g a b=n*sum(zipWith(*)a b)-sum a*sum b`, slope = `g x y/g x x`
- `[..]!!min 7(floor(k+0.9))` beats `last$take(..)`; `!!` binds tighter than `++`; Haskell needs `0.9`, not `.9`
- `main=interact f` + `f s=..where{..}` is 3 shorter than `interact$ \s->let{..}in`
