# TypeScript env on CG

Verified 2026-10-19 with test games on chuck-norris (stderr dumps of CG files):

- compiler: tsc `5.6.2` run by node 20.17.0 (`/codemachine/lib/typescript/internal/ts_compile_classic.sh`): `--target esnext --lib esnext --esModuleInterop --inlineSourceMap --module commonjs --allowJS --baseUrl ./`, `--types` = CG `assert` + CG `readline` + `@types/node` 22.5.5 (undici-types 6.19.8)
- any type error fails the test: `x=readline()` gives `error TS2304: Cannot find name 'x'` and no run. Every variable needs `let`/`var`/`const` (or a declared global); `noImplicitAny`/`strict` are off (no flag), so untyped params are fine
- declared globals: `readline(): string` (CG `index.d.ts`), `assert` (typed as node `assert`, runtime presence unverified), all of `@types/node` (`process`, `require`, `Buffer`...). `print`, `printErr`, `putstr` exist at runtime (javascript polyfill) but are NOT declared: calling them is a type error (inferred from the `--types` list, not tested on CG)
- run: `node -r /codemachine/lib/javascript/internal/polyfill.js -r source-map-support/register /tmp/Answer.js`, cwd `/tmp`, same node/v8/ICU as javascript (see `knowledge/lang/javascript/env.md`): `toSorted` exists, `Object.groupBy` does not at runtime (lib esnext probably still types it, so it type-checks then crashes: unverified)
- local: `agent/tool/lang/typescript/build.sh` uses the same flags and types; same error text on type errors; drift: no `source-map-support` preload (stack traces only), ICU 74.2 vs 75.1
- launcher startup ~38 ms (compile time excluded)
