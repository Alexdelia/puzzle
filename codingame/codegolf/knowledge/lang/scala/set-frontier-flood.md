# Immutable Set BFS flood (surface, 326, CG 100%)

- `ArrayBuffer.foreach` snapshots the size: appending while iterating does not extend the loop (verified). Index `while` loop needed for a growing work list.
- immutable Set of water indices `W` (`"O".r.findAllMatchIn(s).map(_.start).toSet`), per query `var m=W` (persistent: reset is free): `var f=Set(t)&m;while(f.nonEmpty){m--=f;f=f.flatMap(i=>Set(i-1,i+1,i-w,i+w))&m}`, answer `W.size-m.size`. No grid copy, no bounds check, no pad, land start gives 0. Passes 19/19 local.
- `val r="(.+) (.+)".r;for(r(x,y)<-a)` filters only the query lines of an array of lines.
- `scala.io.Source.stdin.getLines.toArray` 39 chars, shortest line array.
