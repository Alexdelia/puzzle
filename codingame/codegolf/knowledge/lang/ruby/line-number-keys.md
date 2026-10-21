# Cheap numeric keys of an input line

Read with `gets` (trailing `\n` kept). For a line holding an integer, these give different values per line at 8 chars, one under `gets.to_i`:

- `gets.oct`: parses octal, stops at the first non-octal digit: `"22\n"` -> 18, `"18\n"` -> 1
- `gets.hex`: `"22\n"` -> 34, `"18\n"` -> 24
- `gets.sum`: byte sum (newline counted): `"22\n"` -> 110, `"18\n"` -> 115 (order flips vs to_i)
- verified ruby 3.1 locally 2026-10-21

Use: pick the one whose difference (and sign) matches what you need, e.g. a repeat count `"s"*(gets.sum*4-373)` (A 67, B 87) is 2 chars under `"s"*(177-gets.to_i*5)`.
String repeat takes a float: `"0 4\n"*1e5` (verified).
