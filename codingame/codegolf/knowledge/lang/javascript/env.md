# Javascript env on CG

Verified 2026-10-19 with test games on chuck-norris (stderr dumps):

- real version is node `v20.17.0` (v8 `11.3.244.8-node.23`), not the 20.9.0 of the help center page: same v8 locally: `toSorted` `findLast` `replaceAll` `at` `isWellFormed` exist, `Object.groupBy` and `Array.fromAsync` do not (checked on local same-v8 node, not on CG)
- run: `node -r /codemachine/lib/javascript/internal/polyfill.js /tmp/Answer.js`, cwd `/tmp`, CommonJS (`require` works), TZ unset = UTC, `LC_ALL=en_US.UTF-8` (Intl locale `en-US`)
- polyfill globals (golf-relevant): `readline()` returns next line without `\n`, `null` at EOF; `print = console.log`; `printErr = console.warn`; `putstr = process.stdout.write` (unbound, calling it bare likely throws, unverified)
- `readline` reads `/dev/stdin` in 256-byte chunks synchronously: mixing it with `require('fs').readFileSync(0)` loses buffered data, unverified
- ICU 75.1 / CLDR 45.0 bundled on CG; local nix build has 74.2 / 44.1: Intl formatting may differ locally, unicode 15.1 same
- local launcher reproduces polyfill (verbatim copy in `agent/tool/lang/javascript/polyfill/`), cwd, TZ, LC_ALL; startup ~62 ms
