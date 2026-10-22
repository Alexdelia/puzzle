# Digit followed by a letter is an invalid literal

Verified locally (OCaml 4.14.2, `agent/tool/lang/ocaml/build.sh`, music-scores cell 2026-10-22).

- `x=0to w`, `w-1do`, `ref 0and`, `q<2then`, `h*7/8in` fail with `Invalid literal 0to` etc.: the lexer reads digits plus following letters as one (bad) literal. A space is needed after a digit before any keyword
- workarounds: end the expression on an identifier or `)` (`h*7/8*w in`, `w and`), loop one step further (`for x=0 to w do` instead of `0 to w-1 do`) when the extra iteration is harmless
- `*!`, `&&!`, `<=!` lex as one operator: `13* !i`, `&& !p` need the space, or reorder (`!i*13`)
