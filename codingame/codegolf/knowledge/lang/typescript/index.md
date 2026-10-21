# Typescript

- [env](env.md): tsc 5.6.2 esnext/commonjs, type errors fail the test, only `readline` declared (not `print`), node 20.17.0 runtime with JS polyfill, verified
- [regex-overlap-merge](regex-overlap-merge.md): ternary reset/skip/regex merge for superstring folds, 124 B, verified
- [ts-ignore-header](ts-ignore-header.md): `//@ts-ignore\n` + one-line JS (undeclared globals, `print`) passes tsc, JS cell + 13 B, verified on CG
- [type-traps-golf](type-traps-golf.md): join``, replace(re,0), Buffer(x), charCodeAt() are type errors; chuck-norris 138
- [readline-any-header](readline-any-header.md): `let L:any=readline` makes all downstream any; header string usable as numeric bound, verified
- [execsync-wrapper](execsync-wrapper.md): `require("child_process").execSync(core,{stdio:[0,1]})`, 52 B overhead, no ts-ignore, verified on CG
- [readline-flat-grid](readline-flat-grid.md): flat char array via readline + push(...R(),0), eval query, 193 on surface, verified
- [ts-ignore-js-port](ts-ignore-js-port.md): `//@ts-ignore` + one-line JS port costs +13 B, beats let/any declarations, verified CG
- [param-as-local](param-as-local.md): arrow param instead of let/ts-ignore, `[...'01234567'].map(i=>readline()+i).sort()[7][1]`, the-descent 69, verified
- [eval-string-header](eval-string-header.md): `eval("js")` = 8 B overhead vs ts-ignore 13 B, bypasses tsc, temperature 74, verified on CG; SLOW for hot loops, see indirect-eval
- [print-spread-for-update-body](print-spread-for-update-body.md): `print(...a)` = console.log spread, loop body folded into the for update, `l=z=` to define a var, verified on CG
- [execsync-perl-core](execsync-perl-core.md): perl flood via execSync, 263 B; CG perl rejects merged variable-length lookbehind; 0-based $#- trick, verified
- [indirect-eval](indirect-eval.md): `eval?.("js")` = indirect eval, global scope, as fast as plain JS (direct eval 10x slower in hot loops), 10 B overhead, verified
