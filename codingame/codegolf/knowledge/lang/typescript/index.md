# Typescript

- [env](env.md): tsc 5.6.2 esnext/commonjs, type errors fail the test, only `readline` declared (not `print`), node 20.17.0 runtime with JS polyfill, verified
- [regex-overlap-merge](regex-overlap-merge.md): ternary reset/skip/regex merge for superstring folds, 124 B, verified
- [ts-ignore-header](ts-ignore-header.md): `//@ts-ignore\n` + one-line JS (undeclared globals, `print`) passes tsc, JS cell + 13 B, verified on CG
- [type-traps-golf](type-traps-golf.md): join``, replace(re,0), Buffer(x), charCodeAt() are type errors; chuck-norris 138
- [readline-any-header](readline-any-header.md): `let L:any=readline` makes all downstream any; header string usable as numeric bound, verified
- [execsync-wrapper](execsync-wrapper.md): `require("child_process").execSync(core,{stdio:[0,1]})`, 52 B overhead, no ts-ignore, verified on CG
