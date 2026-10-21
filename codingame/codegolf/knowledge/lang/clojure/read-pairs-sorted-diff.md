# Clojure: pairs by `(read)`, transpose, sorted-diff median sum (network-cabling 167)

Verified on local referee and CG (network-cabling, 100 %).

- `(for[_(range(read))][(read)(read)])` reads N pairs off `*in*` inside `-main` (35 B, beats slurp+read-string+take-nth, which cost 8+ more)
- `(apply map list P)` transposes a list of pairs; destructure `[[a] b]` of a `(map F cols)` binds `a` as first element of the first result (saves `(first a)`)
- sum of absolute deviations from the median without median or abs: `F = #(map -(sort > %)(sort %))` is non-increasing, its positive entries sum to f(Y) and its first entry is max-min; so `(map F cols)` on [x y] gives range as first of the first and `(filter pos? b)` for y
- `(apply + a seq)` takes extra leading args; `prn` instead of `println` (-3, int output identical)
