# eval string instead of ts-ignore

`eval("<one-line JS>")` costs 8 B (`eval("` + `")`) vs 13 B for `//@ts-ignore\n`: the string is not type-checked, so undeclared `print`, `readline(readline())` (extra arg), ``split` ` ``, string arithmetic in sort comparators all pass. Needs no `"` in the body (use backticks or `'`). Verified CG 2026-10-21: temperature 191 -> 79 (ts-ignore) -> 74 (eval), 100%.
