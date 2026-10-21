# Lua

- [env](env.md): Lua 5.4 (bitwise ops, `//`, `utf8`, goto), 64-bit ints, not LuaJIT, verified
- [os-execute-wrapper](os-execute-wrapper.md): `os.execute"perl ..."` = 13 bytes wrapper, avoid `"` in core via `join$,,`, verified
- [os-execute-long-bracket](os-execute-long-bracket.md): `os.execute[[perl -nle'...']]` when core has `"`, the-resistance 374 native -> 209, verified
