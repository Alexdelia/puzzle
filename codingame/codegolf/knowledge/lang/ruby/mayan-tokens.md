# Ruby: token read, default-arg recursion, digits (verified local mayan-calculation, 185 B)

- `l,h,*a=$<.read.split` works when no row holds spaces; `a.shift(h=h.to_i)`
- recursive lambda with default arg reading input: `v=->n=a.shift.to_i{n<1?0:v[n-h]*20+f.index(a.shift(h))}`, call `v[]`
- `x.send(a[-1],y)` for operator token, `puts n.digits(20).reverse.map{f[_1]}` (puts flattens arrays, `0.digits` is `[0]`)
