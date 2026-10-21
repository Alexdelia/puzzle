# Lua: bulk clamp of non-letters then `&31` index (ascii-art 163, verified local 13/13 + CG)

`t=t:gsub("%A","[")` once (assignment keeps only the string, drops gsub count), then per char `(c:byte()&31)` is 1..26 for both cases and 27 for `[`: slice `r:sub(c*l-l+1,c*l)`. Beats `upper()` + `-65` and the `&95~64` clamp (that one needs `%64` + `math.min`, fails 4 validators when written without the `@`/backtick fix).
Read input with `io.read("n","n","l","l","a")` (second `l` is the text line, first eats the rest of the H line), rows via `r:gmatch"[^\n]+"`. Output `print((t:gsub(".",function(c)...end)))` (double parens drop the count).
