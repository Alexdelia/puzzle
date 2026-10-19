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

- ours Python 117 (2026-10-19); top Python 58
