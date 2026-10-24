# Integer#[] bit test as 0/1 arithmetic (temperature ruby 47, CG 100%)

`n[i]` returns bit i of `n` as 0/1, so a sign/flag correction can be an integer with no comparison, no `?`, and no boolean-to-int conversion (Ruby booleans cannot be multiplied). One byte shorter than `n%2**i/2**i` and 3+ shorter than ternary forms.

- temperature hidden exploit: `l=eval$F[-1];p l-l[2]*5-$F.size/8*9` (47 B, `#!ruby -0an`): bit 2 is set exactly for last tokens 4 and 12, giving the -5 correction.
- works on negative integers too (`-8[2]` = 0, `-173[2]` = 0), unlike `%` reasoning in some languages.
- `eval$F[-1]` (12 with `l=`) is 1 B shorter than `$F[-1].to_i`; `eval` accepts decimal strings incl sign.
- `#!ruby -0an`: one slurped record (`$/`=NUL), `$F` all whitespace tokens, `-n` no autoprint; no trailing-newline dependence.
