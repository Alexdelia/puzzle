# @CompileStatic cuts first-turn latency 5x (verified local 2026-10-21, vox-codei-episode-1)

Measured `main` -> last statement (stderr `runtimeMXBean.uptime` delta, min of 14 local runs): dynamic script reading the first chunk + hash key + decode loop ~45-55 ms, same logic under `@groovy.transform.CompileStatic f(){...}` + `f()` ~10 ms. CG groovy rows of interactive puzzles that timed out at frame 1 at random (see [interactive-first-turn](interactive-first-turn.md)) went 50% -> 74% -> 80% as dynamic latency shrank, then 100% (CG-verified) with CompileStatic.

- shape: `@groovy.transform.CompileStatic f(){def b=new byte[4501];System.in.read(b);for(c in /DATA/[Arrays.hashCode(b)%42..-1].bytes)println c>24?c%12+' '+(c.intdiv(12)-2):'WAIT'}` newline `f()`: costs ~+31 B for the annotation (no import needed), method name needs no `def`
- static mode: undeclared `i=` is an error (`def i`), `(int)it` on a String element does not compile, loop over `.bytes` (ASCII data) gives bytes: arithmetic `c%12`, `c.intdiv(12)` fine
- dynamic costs that vanish: GString println (+20 ms), `Arrays.hashCode` via dynamic dispatch (+30 ms; `b.toList().hashCode()` is the same hash, 15 ms), each new operator call site ~5 ms
- tricks (valid in both modes): `System.in.read(b=new byte[N])` inline assign (dynamic only); `str[k..-1]` replaces `substring(k)`, a NEGATIVE k indexes from the end, so a signed `hash%m` gives two window families and no `abs`/`>>>` is needed
- `System.in.read(b)` returns the first chunk (init + first turn line) on CG: hash of the whole zero-padded buffer is a stable key (java hash identical to python `31*h+byte` over signed bytes)
