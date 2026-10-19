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

- ours Python 1011 (plain reference, 2026-10-19)
