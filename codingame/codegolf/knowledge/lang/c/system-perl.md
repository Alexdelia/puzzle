# system() running a perl one-liner

- `main(){system("perl -anE'...'");}`: wrapper is 19 + 10 bytes (`perl -anE''`), perl 5 is on PATH from C on CG, interactive io works (dont-panic, CG accepted 2026-10-20, 109 bytes, verified)
- core must contain no `"` and no `'`; `$` needs no escape in a C string
- pipe stdout is block buffered: interactive puzzles need a flush, `$|+=<guard>` inside the `if` guard costs 3 bytes (`$|=1;` costs 5)
- set-if-unset hash from a whole line: `%e=(@F[0,1,3,4],%e)` (later pairs win, so existing entries beat the fresh pair), then `$e{$F[0]}`; saves 6 over `for 3,0` with `//=`
- `$F[2]cmp M` is +1 for RIGHT, -1 for LEFT (also NONE: +1)
- `say+(A)*(B)>0?X:Y if COND`: `say(` would be a call, `+` avoids it
- shell fallback: bash core 103 + 19 + `bash -c ''` is longer than perl here
