# readline flat grid (surface 193, verified CG 100%)

`//@ts-ignore` + `w=-~(R=readline)();for(g=[],h=R();h--;)g.push(...R(),0);` builds a flat char array with wall `0` per row (no pad, undefined out of range). Beats `readFileSync`+split (218) by 25 B. Query `eval(R().replace(" ","+w*"))`; loop `for(;n--;print(..))for(i of k=[..])...` needs no braces. Mark by list needs `===` (loose is a timeout).
