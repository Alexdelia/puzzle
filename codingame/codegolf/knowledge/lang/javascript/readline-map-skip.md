# readline in map, float base-20 printer (verified local mayan-calculation, 237 B, rank 4)

- `w=[...Array(+h)].map(R)` reads h lines (`R=readline`); later `w.map(R)` re-reads h fresh lines, array contents ignored: no second array, no loop. `R` ignoring map's extra args is safe
- `putstr` bare throws in the local launcher: `print` only, so output keeping a trailing newline needs glyph strings without one
- float printer without floor: `p=r=>{r>=20&&p(r/20);print(f[r%20|0])}`; `(r/20)%20|0` is the next digit; must use `>=20` not `>19` (19.5 would print a leading 0). Loses exactness above 2^53 (mayan validator 05 fails, CG tests pass)
- recursion on a string count (`g=m=>m?R()+'\n'+g(m-1):''`) works with `m` a string; 251 vs 237 for map version
