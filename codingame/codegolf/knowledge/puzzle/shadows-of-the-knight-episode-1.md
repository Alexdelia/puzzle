# Shadows of the Knight - Episode 1

pretty_id `shadows-of-the-knight-episode-1`, leaderboard `shadows-of-the-knight-episode-1-codesize`, interactive.

## Rules (as referee)

- init lines: `W H`, `N` (jumps), `X0 Y0`. Each turn one line: direction of the bomb from Batman, vertical letter first: `U D L R UR UL DR DL`
- answer `X Y`: Batman jumps there (any window, not only adjacent), jumps left -1
- win: Batman on bomb window after the jump (checked before the jump count: winning on jump N is a win)
- lose: N jumps used, not on bomb ("you are too late"); x or y out of `0..W` / `0..H` ("invalid input. Expected '0 <= x < 4'")
- jumping to the current window is allowed, costs a jump
- bomb fixed during a game, never printed by CG logs (only read off the win frame); same bomb in every log seen, no evasive behavior despite test names "Evasive", "Not there?"
- tests: 4x8 N40, 25x33 N49, 40x60 N32, 1x80 N6, 50x50 N6, 100x100 N7, 9999x9999 N14. Tests 4-7 need near-optimal bisection (binary search with `(lo+hi)//2` uses exactly N on 4-7)

## Output format (probed on CG 2026-10-19)

- `0 2 ` (trailing space): accepted
- ` 3  5` (leading space, double space): fail "Expected 'x y'"
- `3 5 9` (extra token): fail "Expected 'x y'"
- referee model: trim end, split on first single space, both halves must parse as integers

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s and 0.99 s pass, 1.05 s timeout)
- next turns: 150 ms (0.135 s and 0.145 s pass, 0.16 s timeout); statement says 150 ms, right

## Traps

- direction with no horizontal letter means bomb column = current column: a bisection must collapse that axis to the current value, not keep the old range
- the `N` line is never needed by a correct bisection: read and discard

## Shortest known

Method (verified on local referee, 7 tests + 9 validators, python and bash): plain bisection with 4 bound variables named after the direction letters, no position state, start position ignored.

- bounds: `R`=lo x, `D`=lo y start at 0; `L`=hi x, `U`=hi y start at `W`, `H` (read the first line straight into them, no `-1`)
- jump = `((lo+hi)/2, ...)` floor, also `lo+hi>>1`. Ceil fails with these bounds (11/16 in sim)
- each letter `c` of the direction sets bound `c` = current coordinate on its axis (`U`,`D` take y, `L`,`R` take x). Current coordinate = midpoint of the current bounds, so no `x y` state is needed if the midpoint is stored when printed
- no collapse: an axis without letter keeps its bounds (next jump leaves the bomb column, comes back later). Passes every case, validator 08 uses 12 of 14
- ignore `N`, `X0 Y0` AND the first direction: first jump = `(W/2, H/2)`. Passes every case with zero slack on tests 4-7 and validators 3-4 (1x80 N6, 50x50 N6, 100x100 N7, 9999x9999 N14, 80x1 N7, 1x100 N7). So skip lines 2-4 unread, no number parsing beyond `W H`
- output order: print first, then read the direction (the skipped line 4 is the direction relative to the start)

Why short: one assignment per letter, letter = variable name (eval/exec/`$$_`/symbolic refs/bash arithmetic `(($c=...))`), no conditions on direction content.

Letter-to-axis helpers (verified arithmetic): vertical iff `ord%17==0`, or `ord%15%2==0` (U,D -> 0, L,R -> 1); `ord%5` gives U0 L1 R2 D3 with partner `i^3` (one 4-slot array, `b[i]=(b[i]+b[i^3])/2`). `\n` (10) is not `0 mod 17` but is `0 mod 5`: guard it when reading chars.

Packing (verified, python): one value `p = x + y*M`, `M > 2*10000` (e.g. 32768), every letter just gets `p`; `x=(R+L)%M/2`, `y=(D+U)/M/2` are exact (no carry since `Rx+Lx < M`). Useful where "set var named c to one value" is much cheaper than picking the axis. Python 156: longer than picking the axis.

Lowercase trick (verified, bash and python): store the midpoint in `l=r` and `u=d` when printing, then each letter does `c = lowercase(c)` (bash `(($c=${c,}))`).

Wrapper (unverified for this puzzle, leaderboard arithmetic only): bash top 89, C top 111 ~ `main(){system("");}` 19 + bash core with `\n` escape, Python 114 ~ `import os;os.system(...)` + core. See `knowledge/lang/_shared/system-wrapper.md`. Every non-bash cell should compare native with wrapper + shortest bash core.

Lengths reached (local referee only, not CG-verified):

- bash 92, 2 lines: skip lines 2-4 with `head -3>&2` (only lines 2-4 sit in the pipe when it runs) or `read -d_ -t.1 L U _` (timeout keeps partial input, rest goes to `_`); char loop `read -n1 c`, newline char gives empty `c`, then `(($c=${c,},1))` is an arithmetic syntax error -> status 1 -> `||echo` prints the next jump. Error text on stderr is harmless. Tested with local bash 5.3, no 5.2+ feature used (`${d//?/&}` needs 5.2 `patsub_replacement`, CG has 5.1.16: do not use). `head -3` and `read -t` on CG unverified
- python 132 native: `exec(c+'='+'xy'[c in'UD'])` per char after `print(x:=(R+L)//2,y:=(D+U)//2)`; `I=input` alias; `I();I();I()` beats `exec('I();'*3)`
- python: `exec` inside a list comprehension assigns in the comprehension scope: needs a real `for` loop

Rejected:

- using `X0 Y0` and the first direction: costs a number parse (+17 python), skipping is free on all cases
- hi = 10000 for both axes (ignore `W H`): 1x80 N6 cannot work
- step halving (`x += s*d`): needs a sign from letters, longer than bound-per-letter
- python list `"RDLU".find(c)` + 4-slot array: 152 vs 132
- python packed `p`: 156 vs 132
- python tuple per letter `exec(c+'=p')` with `R[0]+L[0]`: 146

bash 88 submitted (local referee 16/16): `read L U;head -3>&2` then `f()((($c=${c,},1))||echo $[l=r=R+L>>1] $[u=d=D+U>>1];read -n1 c;f);f`. Loop trick: `knowledge/lang/bash/subshell-loop.md`. `$[l=r=R+L>>1]` shift precedence works, shorter than `(R+L)/2`.

C (2026-10-20): 116 submitted and CG-validated (#4): `main(){system("bash -c 'read L U;sed 3q>&2;f()((($c=${c,},1))||echo $[l=r=R+L>>1] $[u=d=D+U>>1];read -n1 c;f);f'");}` (bash core 87 with `sed 3q>&2`, 1 byte under `head -3>&2`, + 29 wrapper). Top 111 needs core <= 82. Native C measured 165 (4-slot array, `c%13` index, getchar); perl core inside system 138; dash-only core with sed `\L` lowercase 132 (untested on real dash). Ideas not found to save 5: dropping the skip via `_`-prefixed vars fails (spaces and newlines of lines 2-4 trigger echo).

perl (2026-10-20): 96 submitted and CG-validated (#7), native, no wrapper: `$|=($L,$U)=<>=~/\d+/g;<>;<>;<>;{@m=($R+$L>>1,$D+$U>>1);print"@m\n";$$_=$m[/U|D/]for<>=~/./g;redo}` (literal newline in string). Top 81. Letter-var + mid array beat lowercase vars by 7. Not found: how to skip the gates a -n/-p form needs.

ruby (2026-10-20): native 152 (4-slot `c%5` array, `gets.chomp.bytes`); `exec %q(perl -E'$|=($L,$U)=split$",<>;<>;<>;<>;{@m=($R+$L>>1,$D+$U>>1);say"@m";$$_=$m[/U|D/]for<>=~/./g;redo}')` 111, 16/16 local, not CG-verified. Top 95 = probably `exec"<dash core ~89>"` (6 overhead; sh = dash). Dash-only core found: 130 (`read L U;U=$((U<<15));sed 3q>&2;f()(eval $c=$((x+y*32768))||echo $((x=(R+L)%32768/2)) $((y=(D+U)/65536));c=$(head -c1);f);f` packed p, `eval $c=$p` fails on empty c so `||echo`; `head -c1` reads exactly 1 byte on a pipe; dd bs=1 count=1 is 6 longer). Need dash core <= 89.
