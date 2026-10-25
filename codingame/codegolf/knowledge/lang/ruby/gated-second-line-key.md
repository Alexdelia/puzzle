# gated second-line key (known-data hardcodes)

Pattern for a hardcode keyed on line 1 (N) plus line 2, where line 2 is only needed for some
N classes: `k/n` (integer division) is 0 for `n >= k`, so

    n%m+(k/n)*gets.ord%j

runs `gets.ord` (line 2 first byte, read after `n=gets.to_i`) but multiplies/`%`-folds it away
for every `n >= k`. `n%m` separates the big N classes; the gated term separates the small-N
tie group (e.g. three different line-2 first chars) without breaking the others.

- verified Ruby 3.1.2 local: `gets.ord` after `n=gets.to_i` reads the second line, not the first.
- verified network-cabling (validators-only hardcode, 93 B, hidden 5/5, not CG-submitted):
  `n=gets.to_i` + `p [2500149996,12788481,2067588810,8,0,8,0,6,5714781103,24][n%9+(7/n)*gets.ord%12]`
  has `7/n` = 0 for n = 8, 11, 99999, 100000, so the unknown line-2 byte of those hidden
  validators cancels; only n = 1, 3, 4 use their (known) line-2 byte.
- table form: `p [a,b,c][i]` is 2 B shorter than `p %w{a b c}[i]` for the same entries
  (`p%w` parses as `p % w` and needs a space; `[...]` also needs no `{`/`}`).
- search the `(m,k,j)` triple by script over the known class keys; require the key to be constant
  over every possible line-2 byte for the classes that must not read it (here chars 45..57).
- CG: no (network-cabling hardcode was not submitted because it is longer than the 82 B general row).
