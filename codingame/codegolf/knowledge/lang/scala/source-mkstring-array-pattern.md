# Whole-stdin ints: Source.mkString + Array pattern, tails table, postfix traps

Verified roller-coaster (local 16/16 + CG submit 100%, 2026-10-21):

- `val Array(l,c,n,q@_*)=scala.io.Source.stdin.mkString split "\\D"map(_.toInt)`: header ints + rest as an indexed Seq. `scala.io.Source.stdin.mkString` (30) is 4 shorter than `new String(System.in.readAllBytes)` (34), 3 after the space infix `split` needs. Only `io.Source` (without `scala.`) breaks on CG
- `val l::c::n::q=...toList` is the same length; a List `q` is O(i) to index, fine only for `++`, `tails`, `take`, `sum`
- per-start table over a cyclic array: `(q++q).tails.map(_.take(n)...).toArray` is 3 shorter than `0 until n map(i=>(q++q drop i take n)...)`; `.toArray` needed (`toSeq` of an Iterator is a List, O(i) index)
- `scan(0)(_+_)` is 1 shorter than `scanLeft(0)(_+_)`
- postfix `sum`/`size` before `)` and eta `takeWhile(l>=)` both fail: `postfix operator ... needs to be enabled` (a compile error, not a warning, on 2.13.5). In `x take y map(f).sum`, `.sum` binds to `(f)`: use dots
