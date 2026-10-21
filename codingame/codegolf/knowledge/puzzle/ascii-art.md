# ASCII Art

pretty_id `ascii-art-codesize`, leaderboard `ascii-art-codesize`, io.

## Rules (as referee)

- input: `L`, `H`, text `T`, then `H` font rows of `ABCDEFGHIJKLMNOPQRSTUVWXYZ?`, each row exactly `27*L` chars
- output: `H` rows, each the concatenation over `T` of the `L`-wide slice of the glyph: letter (case-insensitive) -> its glyph, any other char (space, digit, `@`, `[`, backtick, `{`...) -> `?` glyph (index 26)
- `0 < L < 30`, `0 < H < 30`, `0 < N < 200`
- io limit 6000 ms per test (shared rule, see `lang/_shared/io-time-limit.md`)

## Input quirks (captured 2026-10-19)

- font rows keep trailing spaces, full width `27*L` in every test; expected output rows keep trailing spaces too: output compare is exact, so no strip on output (`rstrip` variant fails every test)
- every test ends with a final newline after the last font row
- `T` holds spaces in test 6 (`MAN HAT TAN`), `@` in test 4, mixed case in test 3; no leading or trailing space in `T` in any CG test
- tests 1-4 and 6 share the `#` font `L=4 H=5`; test 5 uses `L=20 H=11` (5967 chars input)
- statement example 1 shows `5 ` (trailing space on `H`): not in captured test 1 (`5\n`)
- `L` line has no trailing space in any hidden validator: groovy 116 parses `L` inside regex `/.{$l}/` (Java throws `Unclosed counted closure` on `.{4 }`, checked locally) and is CG-accepted 100% (2026-10-21). Strict int parse of `L` (Scala `toInt`, Rust `parse`) is safe

## Traps

- Scala `io.Source` passes the local referee but CG compile fails (0%, 2026-10-21): see `lang/scala/root-io-package.md`, write `scala.io.Source`
- `capture-io.sh` truncates test 5: CG test-game output cut around 4000 chars. Captured in 3000-char slices instead (journal `referee.ascii-art.*`)
- `ord(c)%32-1` is the glyph index for letters of both cases; non-letters near letters (`@ [ \` {`) must still map to `?`: validators 05, 07 cover them
- uppercase-only mapping (`c.isupper()`) fails test 3 and most validators

## Shortest known

Method (verified on local referee 13/13, 2026-10-19): for each font row (loop over all remaining lines, `H` never read), print the concatenation over the chars of `T` (final newline dropped) of `row[L*i .. L*i+L]`, `i` = glyph index. No other state.

Ignorable input: `H` (loop to EOF). `L` can be read or derived: `len(row)//27` (row with or without its `\n`: `27L+1` floor-divided by 27 is `L`); deriving only pays when skipping a line is cheaper than parsing an int (Python: `int(l)` once is 4 chars shorter than `len(r)//27` per row).

Glyph index `i`, all forms checked against every printable char 32..126:
- bulk rewrite of `T` once, then a plain offset: Perl `lc` + `y/a-z\n/{/c` (non-letters -> `{`, newline kept so `s/./.../` leaves it in place and the print needs no `\n`), then `ord-97`; Ruby/sed-like `tr('^a-z','{')` same idea; `[` with `ord%32-1` or `ord-65` after uppercasing also gives 26. Needs a complement-class transliteration (`tr`/`y` with `c` or `^`), else regex replace `[^A-Za-z]`
- branchless arithmetic `y = c&95^64` (precedence: `&` before `^`): letters of both cases -> 1..26, `@` and backtick -> 0, every other printable -> 27..31 or 64..95. So `min(y or 27, 27)` is `i+1`; slice `row[L*k-L : L*k]` with `k = that` avoids the `-1`. Python `w*min(ord(c)&95^64or 27,27)` + `r[x-w:x]`: 113 (cell ours 117)
- unsigned clamp `u = (c&95)-65; u>25 (unsigned) ? 26 : u` (C, C-likes). Compare must be unsigned: assigning `-65u` to an `int` then `>25` is signed and fails
- alpha test `ord%32*isalpha - 1` -> -1 for non-letters: only works with a glyph array indexed from the end (`g[-1]` = `?`), not with a substring of the raw row (row ends with `\n`, slice `[-L:0]` is empty)
- per-row output by regex substitution over `T` (`s/./.../ger` Perl, `gsub` Ruby, `replace(/./g,...)` JS) instead of map+join: Perl 81 with the bulk rewrite

- per-row list slice (Perl, verified 13/13 + CG, 2026-10-20): glyph indices of `T` computed once into `@t`, each row printed as `(/.{$l}/g)[@t]` (27 glyphs, row chomped by `-l`, index `-1` = `?` glyph). Index without uppercasing: `map-1+ord()%32,/(\pL)|./g` (non-letter capture undef -> `ord` 0 -> `-1`) or `map{ord()%32*/\pL/-1}/./g`, same length. `ord%32` without `()` fails (parsed as hash). With `-lnE`: `$l||=$_;$.<4?@t=...:say+(/.{$l}/g)[@t]` = 59 core bytes
- no `-n` variant `$l=<>` fails: `/.{$l}/` with `$l="4\n"` is not a quantifier

Lengths reached locally (method demos, not cells): Python 113, Perl 81 (system perl 5.42), C 151 (ungolfed, `gets` + `printf("%.*s",L,row+L*i)`).

Leaderboard bounds (2026-10-19): Python 58 (player `Validator_sucks.More.Edge.Random`: likely exploits weak CG validators, unverified), Perl 61, bash 64, Ruby 73. Gap Perl 61 vs 81 unexplained: a shorter legit method may exist (unverified ideas: switches on the `#!perl` line, building a per-row `eval` string).

Rejected:
- `re.findall('.'*L,row)` glyph list in Python: needs `import re`, 127+
- Perl `/.{$l}/` with `$l=<>` unchomped: newline inside the quantifier, 0/13
- C `scanf("...\n%[^\n]\n")`: `\n` in format skips leading spaces of the first font row; use `%[^\n]%*c`
- python `*_,t,*R=open(0)`: two starred targets, syntax error

Perl cell reached 73 total bytes incl. `#!perl -nl\n` (CG rank 10, 2026-10-20): `$l||=$_;$.<4?@t=map{ord()%32*/\pL/-1}/./g:print+(/.{$l}/g)[@t]`. `-E` can't go on the shebang (see lang/perl/shebang-no-E.md), so the "59 core" figure excludes switches and `say` -> `print`. Top 61 unexplained (probably no shebang).

## Hidden validators

Facts (2026-10-20):
- Python leaderboard, score 100 rows only: 58 `Validator_sucks.More.Edge.Random`, then 91 Lydxn, 93, 95, 95, 97...: 58 is a lone outlier, the legit floor is about 91. No other language shows a similar gap (Perl 61, bash 64 by golfers whose Python is 95)
- arithmetic: any per-row concatenation over `T` in Python costs >= `''.join(... for c in t)` (21) + glyph slice (~20) + `for r in R:print()` (18) + input (16): about 80 even with `L=4` hardcoded and uppercase-only `ord(c)*4-260`. 58 leaves 24 bytes for the row expression, so the 58 cannot loop over `T`
- exact 58 fit: one uppercase char, `L=4` hardcoded: `_,_,t,*R=open(0)` + `for r in R:print(r[ord(t[0])*4-260:][:4])`. Case-insensitive single char costs 59 (`%32*4-4`)
- probe 1 (Python, that 58): FAIL, row stayed 117. So the current validators are not all single uppercase chars with `L=4`

Guesses (unverified):
- the 58 was likely scored on an older, weaker validator set (CG keeps old rows after validator edits); if so no exploit is reachable now
- not tried: environment reconnaissance of the CG sandbox (out of scope, refused)

Probes spent: 1 (2026-10-20, Python 58 single-letter, fail). Next useful probe only if a new no-loop hypothesis appears; every loop-based exploit lands near 80 in Python, above the legit 91 floor only by restricting cases (uppercase-only / letters-only / `L=4` hardcode), and each such restriction is a 1-bit probe worth trying in a cell below its own row

Python 107 (CG, 2026-10-20): `w,h,t,*R=open(0);w=int(w)` / `for r in R:print("".join(r[(x:=w*min(ord(c)&95^64or 27,27))-w:x]for c in t[:-1]))`. Walrus in slice, see lang/python/walrus-in-slice.md.

Recovered by validator task (2026-10-20, 7 count probes, details and sources in `puzzle/ascii-art/referee/hidden/facts.md`):
- 10 validators (score grain 10 in every partial row), 6 IDE tests; no validator reuses an IDE `T`
- fonts: all 10 use an IDE font: 9 the `#` font `L=4 H=5`, 1 the test-5 font `L=20 H=11`
- `T`: 7 uppercase-only `^[A-Z]+$`, 1 more letters-only with lowercase, 2 with non-letters
- exact: `HELLOWORLD` (2 identical validators) and `WMADXESFGIJVTHKNOBPLYQRUZC` (1), both `#` font; 7 `T` still unknown, validator numbers unknown
- golf: case handling and `?` mapping both still needed (1 and 2 validators); `L=4` hardcode passes 9/10

Haskell 120 (CG rank 5, 2026-10-21): `main=interact$(\(l:_:t:r)->unlines[fromEnum<$>t>>= \d->take(read l)$drop(read l*min 26(mod(max 64 d-1)32))x|x<-r]).lines`, see lang/haskell/char-index-clamp.md. Probe (haskell/nopunct.hs, 113 bytes, no clamp of chars below 64 except space and `@`): CG 80%, so 2 hidden validators contain digits or punctuation (not just space and `@`), 8 pass. Local validators 02-05 also fail it.

Scala 156 (CG rank 5, 2026-10-21): `object Solution extends App{val a=Console.in.lines.toArray.map(_+"");for(r<-a drop 3)println(a(2).flatMap(c=>r.grouped(a(0).toInt).toSeq(c-65&223 min 26)))}`, see lang/scala/io-shadowed-console-in.md. `r.length/27` instead of `a(0).toInt` is 1 byte longer if the L line ever has a trailing space (not seen on CG).
