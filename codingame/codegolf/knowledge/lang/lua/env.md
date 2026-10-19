# Lua env on CG

Verified 2026-10-19 with test games on chuck-norris:

- `_VERSION` = `Lua 5.4` (patch not exposed; CG help lists 5.4.4, local nix is 5.4.4): integer division `//`, bitwise ops `& | ~ >> <<`, `utf8` lib, `<const>`/`<close>` attribs, `goto` all available
- 64-bit integers (`math.maxinteger` = 9223372036854775807), not LuaJIT (`jit` nil)
- `io.read("n","n")` reads 2 space-separated ints (local smoke; across lines unverified)
- stderr shows in test-games `output` but is not judged
- `package.path` differs (CG `/usr/local/...`, local nix relative): irrelevant, no extra libs on CG
- local launcher startup ~9 ms
