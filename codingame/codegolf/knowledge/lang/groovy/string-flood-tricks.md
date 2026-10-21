# Groovy: input/regex/index tricks for grid floods (verified local 3.0.8, surface 2026-10-21)

- `System.in.text.split('\n',3)[2]`: drops the 2 header lines in one go (String.split with limit), rest keeps its newlines
- `t.findAll(/regex/){...}`: closure runs per match with `it` = match string, no `.each` needed (-5 bytes)
- `(x,y)=it.split()*.toLong()` instead of `toInteger` (-3 bytes): Long indexes work for `List.getAt` and `+ - *` with Integer, ~15% slower on a 4M-push loop
- `list[i]` negative index wraps from the end (throws only below -size), index past the end gives null: no bound pad needed for a flat list grid, but the wrap lands on the last row (spurious link top row <-> bottom row when the tail after the grid is shorter than a row)
- `t.toList()` of a String = list of 1-char Strings (1M allocations); `[*t]` does not work (cannot spread String); `[*t.chars]` works but Character has no `.size()`
- TRAP: `/.+ .+/` over rows of 1000 equal chars is quadratic (every start position scans the row): costs ~1 s on a 1M-char grid, use `/\d+ .+/`
- TRAP: `w=~x` parses as the `=~` find operator, needs a space (`w= ~x`)
- TRAP: `for(i in list)` while appending throws ConcurrentModificationException: use `for(j=0;(i=k[j++])!=null;)` (0 is falsy, so `i=k[j++]` alone breaks on cell 0)
- `evaluate(str)` inside a script shares the binding but compiles each call: 300 calls = timeout
- speed: dynamic Groovy flood costs ~1 us per cell visit-with-4-pushes; 1M all-water cells ~4-5 s on a pinned loaded core. Typed int[] version ~2x faster but much longer
