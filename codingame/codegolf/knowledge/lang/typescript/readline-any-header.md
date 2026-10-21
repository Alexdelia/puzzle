# readline as any, header string as bound

Verified local 2026-10-21 (network-cabling 186 bytes, 21/21): `let L:any=readline,l,n` makes every later value `any`, so `n=L()` (string) works in `i<n` and ``l.split` ` `` destructures without type errors. `s+=y` with string `y` concatenates: use `s-=c?y:-y` to force numbers.
