# zipAll to merge two letter columns of unequal length

Verified 2026-10-21 (power-of-thor, local 8/8, CG 100%, scala 167):

- `"S"*n` with negative `n` is `""`, so `"S"*(b-d)+"N"*(d-b)` is the whole vertical run without sign or abs
- `for((p,q)<-v zipAll(h,"",""))println(""+p+q)` prints line k = `v(k)` then `h(k)`, missing side empty; `zipAll` infers `(Any,Any)`, `""+` turns them back into text
- 6 shorter than `v zipAll(h,"","")foreach(p=>println(""+p._1+p._2))`, 8 shorter than a `for(i<-0 to 40)` counter with `(..drop i take 1)` per axis
- `val Seq(a,b,c,d)=<Array>` does not compile (scrutinee incompatible), keep `val Array(...)`
