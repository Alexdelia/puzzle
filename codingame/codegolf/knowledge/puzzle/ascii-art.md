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

## Traps

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
