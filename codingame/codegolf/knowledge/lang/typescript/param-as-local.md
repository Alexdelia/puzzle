# map param as the only variable (no let, no ts-ignore)

Verified local 15/15 (the-descent 69 B, CG accepted): `for(;;)console.log([...'01234567'].map(i=>readline()+i).sort()[7][1])`. Arrow param `i` declares the loop index for free, beats `//@ts-ignore`+JS (71) and `for(let m,h,i;;...)` (73). String spread `[...'01234567']` gives index chars; `readline()+i` string key sorts like int.
