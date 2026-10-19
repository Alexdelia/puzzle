# Mayan Calculation

pretty_id `mayan-calculation-codesize`, leaderboard `mayan-calculation-codesize`, io.

## Rules (as referee)

- input: `L H`, then `H` font rows of the 20 numerals side by side (`20*L` chars each), `S1`, `S1` rows of the first number, `S2`, `S2` rows, operator line `+ - * /`
- a number = `S/H` stacked glyphs of `H` rows each, most significant on top; glyph -> digit by match against the font
- output: result in base 20, same glyphs, most significant on top, `H` rows per digit, no blank line between digits; result `0` = one `0` glyph
- `/` is exact (remainder 0); `-` never negative in CG tests
- io limit 6000 ms per test (shared rule, see `lang/_shared/io-time-limit.md`)

## Input quirks (captured 2026-10-19)

- every test ends with a final newline after the operator line
- only line 1 (`L H`) holds a space in every test: fonts use `.o_` (tests 1, 8-10 read, 2-7 not inspected), digits/letters `0-9A-J` with `L=H=1` (test 11), `.X` with `L=H=11` (test 12)
- glyphs differ only by layout, several share whole rows (e.g. all of `5..19` share row 2 in the `.o_` font): match the whole glyph, not one row
- the `.o_` font of tests 1-10 has `....` for 5 row 0 and `____` lines for bars; the statement example font differs (`____` in row 0 from digit 10)
- test 9 = product by zero, output one `0` glyph; test 10 = number with a `0` digit in the middle
- largest CG result is small (test 8 is 4 digits x 5 digits); statement caps inputs at `2^63`, not results

## Validators

- hand-built: zero result of `a-a` and `0*n`, carry chain `19 19 19 19 + 1`, borrow chain `8000-1` in a `3x2` font, exact division with `0` middle digit, product `4*20^14+20^8+20+1` (above `2^62`, fits signed 64-bit), 1x1 fonts
- no result reaches `2^63`: an int64 solution is not rejected locally for something CG may never test

## Traps

- a base-20 loop `while r: ...` prints nothing for result `0`: fails test 9 and validators 01, 04, 09
- `eval` float `/` passes every CG test (small values); keep `//` or int division to be safe on big quotients

## Shortest known

Method (2026-10-19, algorithm worker), 5 parts, each proven on the local referee (Python, 21/21 tests + validators):

1. font: digit `d` = list of `H` row slices `row[d*L:d*L+L]` (or: scan each font row into `L`-char chunks, transpose -> 20 lists of `H` strings; Ruby `scan(/.{#{l}}/)` + `transpose`). Keep the glyph as a list of row strings: list equality needs no join, and output prints the same list
2. digit lookup: `index` of the `H`-row block in that list of 20 glyphs (whole-glyph equality, rows alone are ambiguous)
3. number value: Horner, read on the fly. Recursion on the remaining line count `n`: `v(n) = n ? v(n-H)*20 + index(next H lines) : 0`. Left-to-right evaluation makes `v(n-H)` read the upper glyphs first, so no list, no loop, no `//H`: pass the raw `S` line count straight in. Languages with `each_slice`/chunk + `inject`/`reduce` get the same in one chain
4. operator: `eval` of `a<op>b` where the language has it (Python result needs `int()` for `/`; Ruby: `eval "a"+gets+"b"` keeps the newline after the operator, a binary operator at line end continues the expression, unverified locally: no `CG_RUBY` in shell). Without eval: `c=op char`, `c%2 ? (c>46 ? a/b : a+(44-c)*b) : a*b` - `+` and `-` share `a+(44-c)*b` (43 -> +b, 45 -> -b), `*`=42 is the only even code, `/`=47 the only one >46. Python 21/21. Beware `and/or` instead of ternary: `a-a=0` falls through to `a*b`
5. output: base 20, most significant first, zero must print one glyph. Shortest: recursion `p(r){ r>19 && p(r/20); print glyph r%20 }` - prints `0` for free, no reversal, no digit list. Native radix (Ruby `to_s(20)` + `to_i(20)` per char, JS `toString(20)` + `parseInt(c,20)`, Java `Long.toString(r,20)`) also prints `0` for zero and may beat the recursion; bash `bc` with `obase=20` prints digits as space separated 2-digit decimals (unverified, no bc in shell)

What can be ignored: nothing structural. `S1`/`S2` counts are needed (L=H=1 test 11 has digit glyphs `0-9A-J`, a count line looks like a glyph row). Final newline of input never needs handling with token or line reads.

Lengths reached (local, all 21 pass):

- Python 256: `input()` per line, font slices, recursive Horner lambda on the line count, `eval` of an f-string `{a}%s{b}` formatted with the operator (f-string evaluates both numbers before `%` reads the operator line: order kept), `int()` around it, recursive printer `r>19and p(r//20)or print(...)`. Top Python is 183: a trick worth ~70 is still unknown
- Python 266: all-tokens `open(0).read().split()` + list slicing by `S1`: longer than `input()` here (two `int()` for `L,H` + slice arithmetic)
- Python 296: no-eval operator formula instead of `eval` (+40 in Python; the formula is for languages without eval)

Rejected:

- joined glyph strings + `str.find` in the concatenated font: misaligned matches possible unless a separator is inserted, and the joins cost more than list equality
- `zip(*[iter(row)]*L)` / `re.findall` chunking in Python: yields tuples, block must be converted, longer than slicing
- `while` digit loop with `[]==o or r` guard: about 3 longer than the recursive printer in Python (estimate, not measured in isolation)
- doubling the operator (`a//b`): `a--b`, `a**b` break `-` and `*`
