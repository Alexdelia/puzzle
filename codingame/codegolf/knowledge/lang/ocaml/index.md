# OCaml

- [env](env.md): 4.14.2 ocamlopt native, stdlib only (Str/Unix fail at link), verified
- [scanf-read-int-mix](scanf-read-int-mix.md): `%d` skips no whitespace (use `" %d"`), `read_int()` then Scanf works (shared buffer), `List.(...)` around the program, verified
- [system-wrapper](system-wrapper.md): whole program = bare `Sys.command"perl ..."` (13 B overhead, no `let _=`), `{|..|}` literal +2 B takes `"` and `\` raw, perl on CG OCaml box, verified
- [substring-test](substring-test.md): `try sub u i(length t)=t||m t u(i+1)with _->false` = t in u, `let rec..and` chain, verified
- [toplevel-expr](toplevel-expr.md): bare `Sys.command"..."` as whole file compiles (int toplevel, no `let _=`), wrapper `Sys.command"bash -c '<core>'"` = 23, verified
- [scanf-abs](scanf-abs.md): `Scanf.scanf" %d"abs` reads an int, first format needs no leading space, `1to` is an invalid literal, verified
- [fold-make-loop](fold-make-loop.md): `fold_left(fun v _->..)0(make c 0)` = shortest c-times loop, pack pair state in one int, verified
- [printf-star-is-width](printf-star-is-width.md): `%.*s` pads (width), no precision truncation for strings, verified
- [local-letter-helper](local-letter-helper.md): `let r l p q=if i+q<p then l else""in` + curried `r"S"b d^...`, `for i=0 to 99999` keepalive, verified CG
- [digit-letter-literal](digit-letter-literal.md): `0to`, `1do`, `8and`, `2then` are invalid literals, space needed after a digit; `*!`/`&&!` lex as one operator, verified
- [rle-decode-array-fill](rle-decode-array-fill.md): RLE decode via `Array.fill` with toggled colour, program inside the EOF handler; `s^String.make` concat is quadratic (timeout at 1M), verified CG
