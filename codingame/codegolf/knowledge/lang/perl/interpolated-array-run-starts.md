# interpolated array run starts

Verified on CG (music-scores perl 164, 2026-10-22).

- sparse array `@f` (undef = gap): `"@f"` joins undef as empty, so a gap shows as two spaces; `"@f"=~/  (\S+)/g` = first element of every run, no previous-flag loop (`$p||$_&&push@o,..;$p=$_`)
- store the final output token in the array at the first hit (`cond and$f[$k]||=TOKEN for..`) so the regex returns output directly, no `map`
- print with `$,=$";print LIST` (same length as `print"@{[LIST]}"`); a first element at index 0 would be missed (no leading double space)
