# Haskell: perl -E wrapper, say and q()

`import System.Cmd;main=system"perl -E'CORE'"` costs 41 bytes around the Perl core (18 import + 12 `main=system"` + 9 `perl -E'` + 2 `'"`).

- `-E` enables `say`: `say LIST` is 2 shorter than `print LIST,"\n"`-style cores and needs no `\n` (a `\n` inside a Haskell string costs 2 bytes)
- avoid `\"` escapes: `q( 4)` for a literal string (5 bytes, same as `\" 4\"` minus 1)
- interactive: Haskell never touches stdin, perl inherits the pipes; a perl core that prints far ahead (`for<>*5..1e5`) blocks the pipe and stays alive
- verified mars-lander-episode-3 2026-10-22: `perl -E'say$_<170?11:$_<185?-89:0,q( 4)for<>*5..1e5'` = 83, CG 100% #1 (top was 84, native Haskell best 90). GHC warns System.Cmd deprecated, compiles
- rule of thumb: Haskell row > perl row + 41 (minus say/escape adjustments) means the wrapper wins
