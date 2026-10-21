# print(...a) and loop body in the for update (JS/TS with ts-ignore)

Verified local 2026-10-21 (shadows-of-the-knight-episode-2 TypeScript 280 -> 257, 46/46, CG 100%):

- `print` is `console.log` in the polyfill (`global.print = console.log`, `agent/tool/lang/javascript/polyfill/polyfill.js`): `print(...a)` prints `a[0] a[1]` space-separated (a.join` ` costs 5 B more), `print(m,l)` likewise. Values may be strings or numbers
- `for(init;cond;stmt1,stmt2,...)body` with a one-expression body: the check runs first, then body, then update: moving the whole loop body into comma expressions of the update/body saves the `{}` braces; `if(c)x;` becomes `c&&(x),`
- an undeclared variable read throws (`m=l` before `l` is ever assigned): chain-assign `l=z=readline()...` (2 B) to define it
- `for(i in z)` over a 2-element array is 3 B shorter than `for(i of[0,1])`, `i` is the string key (fine for indexing)
