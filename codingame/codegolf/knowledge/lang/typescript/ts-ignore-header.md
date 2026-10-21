# `//@ts-ignore` first line makes JS-style golf legal in TS

Verified 2026-10-21 locally (ascii-art, 13/13) and on CG (ascii-art TypeScript 120, rank 11): file = `//@ts-ignore\n` + a one-line JS solution using undeclared globals (`t=(R=readline)(R(l=R()))`, `print(...)`, implicit `i`, `readline` with arguments). `ts-ignore` silences every error on the next line, so the whole program must be one line. 13 B overhead; `//@ts-nocheck` works too but is 1 B longer. Typed alternative costs more: `let` declarations, `console.log` (+6 over `print`), `readline` takes no args in the declared type.

Consequence: a TS cell is the JS cell + 13 B; reuse `lang/javascript/` tricks (`print` exists at runtime via the polyfill, confirmed on CG).

Superseded for one-line bodies (checked 2026-10-21 on all 11 committed ts-ignore solutions): `eval("<body>")` (8 B, see [eval-string-header](eval-string-header.md)) is 4-5 B shorter than `//@ts-ignore\n` (13 B) with the same local pass set on 10 of 11 (the-resistance 13/18 vs 14/18, check). Escape `\\` and the chosen quote, pick the quote absent from the body.
