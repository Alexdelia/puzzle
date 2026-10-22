# Spawn another interpreter from C++

Verified on CG 2026-10-22 (C++, g++ 11.2), generators in `agent/wrap/cpp/`:

- `#import<ios>` + newline + `main(){system(<literal>);}` = literal + 31. C++ needs a declaration of `system`: with `$CG_CPP` g++ 11.2 `<ios>` is the shortest header that exposes it (`<string>`, `<regex>`, `<vector>`, `<cstdlib>` also do; `<new>`, `<any>`, `<map>`, `<set>`, `<list>`, `<array>`, `<tuple>` do not). CG accepts it (all test games below compile).
- `system` runs `/bin/sh` = dash, waits, child inherits stdin/stdout: interactive works with plain `main` (the-descent-codesize test 1, game 910947672, perl core). No Player/Solution entry point.
- literal: `"..."` with C escapes (`\\`, `\"`, `\n`), or raw `R"(...)"` (+3, core verbatim, newlines kept) when the core has many `"`/`\`/newlines.
- box (`command -v` probe, temperature-code-golf test 1): perl, python3, python, bash, `/bin/lua`, `/bin/php` on PATH; ruby only `/usr/local/bin/ruby`; no node/nodejs. Generated perl/python3/sh/lua/php/ruby wrappers pass temperature-code-golf test 2.
- CG runs C++ under gdb: the console output starts with `[Detaching after vfork from child process 30]` after a `system` call. Not on judged stdout (tests still succeed).
- dash trap: bash core `read;` fails under dash (`read: arg count`); `read a;` (+2) beats `bash -c '...'` (+10). Generator emits that variant.
