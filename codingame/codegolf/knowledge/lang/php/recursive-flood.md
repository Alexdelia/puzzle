# PHP recursive flood fill on a flat string (surface, 217 bytes, 100% on CG hidden validators)

- userland recursion has no depth limit in PHP 7.3 (VM stack on heap): 360k-deep flood is fine, no C stack overflow
- `function f($i){global$h,$w;return$h[$i]==O?($h[$i]=1)+f($i-1)+f($i+1)+f($i-$w)+f($i+$w):0;}`: `($h[$i]=1)` marks and counts, string offset write, negative offset reads from the end (harmless), out of range reads notice only
- header strip: `[$w,,$s]=explode("\n",`cat`,3);$w++;` keeps the rows flat with their `\n` as walls; `$h=$s` per query copies 1 MB cheaply
- queries: `preg_match_all("/(.+) (.+)/",$s,$m,2);foreach($m as[,$x,$y])`
- cost: ~1.2 s per 300k-cell flood, 6 s for 1M cells; the array version with `array_push($k,...)` and `foreach($k as&$i)` (reference iteration sees appended items) is as slow
- marking array cells with the work list by reference needs `unset($k)` per query (else the assignment writes through the old reference)
