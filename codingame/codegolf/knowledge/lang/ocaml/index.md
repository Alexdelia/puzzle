# OCaml

- [env](env.md): 4.14.2 ocamlopt native, stdlib only (Str/Unix fail at link), verified
- [scanf-read-int-mix](scanf-read-int-mix.md): `%d` skips no whitespace (use `" %d"`), `read_int()` then Scanf works (shared buffer), `List.(...)` around the program, verified
- [system-wrapper](system-wrapper.md): whole program = bare `Sys.command"perl ..."` (13 B overhead, no `let _=`), perl and python3 on CG OCaml box, verified
