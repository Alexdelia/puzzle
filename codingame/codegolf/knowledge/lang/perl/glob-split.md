# glob as whitespace split

`glob EXPR` splits its argument on whitespace and returns words without wildcards unchanged, so `($a,$b)=glob<>` = `($a,$b)=<>=~/\d+/g` minus 4 bytes (trailing newline dropped too). Several lines at once: `glob<>.<>.<>.<>` reads exactly 4 lines (left-to-right `.`), safe in interactive puzzles (no list-context `<>`), returns all their words. Header + skip 3 lines: `$|=($L,$U)=glob<>.<>.<>.<>;` vs `$|=($L,$U)=<>=~/\d+/g;<>;<>;<>;` = -4.

Traps: words with `* ? [ { ~` are expanded as patterns (fine for numbers and letters). `glob` alone uses `$_` (-n forms).

Verified: shadows-of-the-knight-episode-1 perl 92, local 16/16, CG-accepted 2026-10-22 (submit.sh).
