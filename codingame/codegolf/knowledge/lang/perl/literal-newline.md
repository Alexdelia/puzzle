# Literal newline instead of `\n`

- inside a regex or a double-quoted string, a raw newline byte matches/means `\n` and costs 1 byte instead of 2, verified (`/ $`<newline>/` in dwarfs, CG accepted under bash `perl -pe`)
- not with `/x` (whitespace ignored there)
