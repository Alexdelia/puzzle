# Glyph index without Data.Char (ascii-art, verified 13/13 and CG 100%, 2026-10-21)

`d` = `fromEnum c`. `min 26(mod(max 64 d-1)32)`: letters of both cases -> 0..25, every printable non-letter -> 26.
- `mod(d-1)32` folds case (65 and 97 both -> 0) and sends `@`, space to 31; `max 64 d` sends anything below 64 (digits, punctuation) to 63 -> 31; `min 26` clamps `[`..backtick and `{`..`~`.
- without `max 64`: 113 bytes total but CG scores 80% (2 hidden validators carry digits or punctuation below 64)
- `>>=\` lexes as one operator: write `>>= \d->`
- `fromEnum<$>t>>= \d->body` (list monad) replaces `concat[..|d<-map fromEnum t]`
