# `gets` with a String separator crosses lines (Ruby 3.1.2, verified locally)

`gets` takes a *separator*, not only a length: `Kernel#gets(" ")` / `gets" "` reads from the
current position **through the first occurrence of `" "`, crossing newlines**:
`"100\n10 240000\n11 1\n"` -> `"100\n10 "`, then the next `gets" "` -> `"240000\n11 "`.
This is not the same as `gets(32)` (an Integer arg = one line capped at n chars, never crossing
`\n`), see [gets-limit-vs-read](gets-limit-vs-read.md).

- bare `gets" "` costs 6 B, `(gets" ").sum` 12 B: a cross-line prefix key 2 B under
  `$<.read(5).sum` (14 B). `(gets"X")` = 8+|X| B, so a 2-byte separator like `(gets"34")` is 10 B.
- **parse trap**: an unparenthesised method call is taken as the argument, not applied to the
  result: `gets" ".sum` parses as `gets(" ".sum)` = `gets(32)` (a 32-char line read) and
  `gets" ".to_i` as `gets(0)` (empty). Always write `(gets" ").sum` or `gets(" ").sum`.
- the read is taken from the *known* input only up to the separator, so a separator that first
  occurs inside a noisy field is not a usable key (puzzle-dependent).
- `String#sum` of the read is additive, so two inputs with equal line-1 byte sums collide no
  matter where the read stops (needs a non-additive hash); `.hex`/`.oct`/`.to_i` parse the
  leading number only and stop at the newline in the read, so they cannot see bytes after `\n`.
