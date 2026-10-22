# OCaml

- [env](env.md): 4.14.2 ocamlopt native, stdlib only (Str/Unix fail at link), verified
- [scanf-read-int-mix](scanf-read-int-mix.md): `%d` skips no whitespace (use `" %d"`), `read_int()` then Scanf works (shared buffer), `List.(...)` around the program, verified
- [system-wrapper](system-wrapper.md): whole program = bare `Sys.command"perl ..."` (13 B overhead, no `let _=`), perl and python3 on CG OCaml box, verified
- [substring-test](substring-test.md): `try sub u i(length t)=t||m t u(i+1)with _->false` = t in u, `let rec..and` chain, verified
- [toplevel-expr](toplevel-expr.md): bare `Sys.command"..."` as whole file compiles (int toplevel, no `let _=`), wrapper `Sys.command"bash -c '<core>'"` = 23, verified
- [scanf-abs](scanf-abs.md): `Scanf.scanf" %d"abs` reads an int, first format needs no leading space, `1to` is an invalid literal, verified
- [fold-make-loop](fold-make-loop.md): `fold_left(fun v _->..)0(make c 0)` = shortest c-times loop, pack pair state in one int, verified
