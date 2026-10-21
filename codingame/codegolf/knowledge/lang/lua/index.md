# Lua

- [env](env.md): Lua 5.4 (bitwise ops, `//`, `utf8`, goto), 64-bit ints, not LuaJIT, verified
- [os-execute-wrapper](os-execute-wrapper.md): `os.execute"perl ..."` = 13 bytes wrapper, avoid `"` in core via `join$,,`, verified
- [os-execute-long-bracket](os-execute-long-bracket.md): `os.execute[[perl -nle'...']]` when core has `"`, the-resistance 374 native -> 209, verified
- [golf-tricks](golf-tricks.md): io.read n,n,a; gsub with s.rep; s.char multi; byte-to-char mod trick
- [perl-sort-pop-pairs](perl-sort-pop-pairs.md): sorted pair-ends via pop in foreach, int32 comparator trap, verified
- [gsub-bulk-clamp](gsub-bulk-clamp.md): `t:gsub("%A","[")` then `byte()&31` letter index, ascii-art 163, verified
- [offset-indexed-data](offset-indexed-data.md): hardcode table as one string indexed by key offset, `:sub(key)` instead of separators, vox-codei-episode-1 217 -> 192, verified
- [rep-sep-lines](rep-sep-lines.md): `print(s:rep(n,"\n"))` = s on n lines, -4 vs for loop, verified
- [print-tab-separator](print-tab-separator.md): print(a,b) joins with TAB; numeral before or/and needs a space
