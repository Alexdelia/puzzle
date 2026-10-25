# `-F` takes a Regexp (escapes, classes, zero-width), not just one character

Ruby's `-F` shebang switch sets `$;` to a **Regexp** built from the argument, so `-a`
splits with regex semantics.  Verified local Ruby 3.1.2 (`ruby -0an -F... -e 'p $F'`):

- `-F\s` -> `/\s/`: awk-like whitespace split (same as default for clean input)
- `-F\n` -> `/\n/`: one field per line
- `-F[ -]` -> `/[ -]/`: split on space or minus (minus is consumed, sign lost)
- `-F(?=-)` -> zero-width split *before* each minus: `"5\n7 5 9 -1 4\n"` ->
  `["5\n7 5 9 ", "-1 4\n"]`, so the minus stays with the following field
- `-F(?<=\d) ` -> `["5\n7", "5", "9", "-1", "4\n"]` (space consumed after a digit,
  newline kept at the end of the previous field)
- `-F\A.*\n` -> removes line 1 and leaves a leading empty field:
  `["", "7 5 9 -1 4\n"]` (split keeps the empty field)
- `-F''` (empty) sets `$;` to nil -> default whitespace split

Cost: the shebang grows by `2 + len(separator)` bytes over `#!ruby -0an`
(`F` plus the pattern plus the byte it displaces relative to no -F).  A separator
matches zero-width positions too (`(?=...)`, `(?<=...)`, `\b`, `\B`), so arbitrary
re-tokenisation is possible, but the `$;` regexp cannot reference the input.

Not useful on temperature: every answer is a token of line 2 and separators that
preserve the minus sign leave all tokens (default whitespace already does), while ones
that consume `-` destroy negative answers.  Keep in mind for puzzles where a sign or a
delimiter should be detached/kept with the next field.
