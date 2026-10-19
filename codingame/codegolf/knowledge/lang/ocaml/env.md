# OCaml runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- OCaml 4.14.2, native code (`Sys.backend_type` = Native), 63-bit `int`, source compiled as `/tmp/Answer.ml` to `/tmp/Answer`
- only the stdlib is linked: `Str` and `Unix` typecheck (their `.cmi` sit in the stdlib dir) but fail at link with `No implementations provided for the following modules: Str`. No regexps, no `Unix`
- warnings do not fail the build (unused variable compiles)
- no `-unsafe`: out-of-range array access raises `Invalid_argument`
- stderr is shown in the test-game `output` but not judged
- local: `agent/tool/lang/ocaml/build.sh` = plain `ocamlopt -o main main.ml`, same version, same Str/Unix link error, same bounds checks
