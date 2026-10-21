# Regression slope with grouped(2) rows

Verified (blunder-episode-3, CG 100%, 304 bytes):

- `new String(System.in.readAllBytes).split("\\s").tail.map(x=>math.log(x.toDouble)).grouped(2).toSeq` = rows `p(0),p(1)` (log n, log t); `split("\\s")` is enough when no double whitespace
- slope = `f(1)/f(0)` with `m=mean of p(0)`, `def f(i:Int)=a.map(p=>(p(0)-m)*p(i)).sum` (sum of centred x times x equals centred x squared)
- table index: `"a,b,c".split(",")(i min 7)` beats `Seq(..).lift(i).getOrElse`; fold the shared `O(`/`)` outside the string
- `submit.sh` refuses length >= a 0% row; `--probe` with a passing solution still replaced the row
