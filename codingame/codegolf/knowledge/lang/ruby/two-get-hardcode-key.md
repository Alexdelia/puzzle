# Two-`gets` hardcode key (line 1 and line 2), verified CG

When line 1 and line 2 each hold a number that matters (blunder-episode-3: N then `n t`):
`gets.to_i%16^gets.to_i%9` reads both lines in one index expression (24 B, left-to-right
`gets` order), values distinct on the target inputs. Index a `%w` table directly; unused
slots are filled with the shortest body (`n`, 2 B each) and the table ordered by key value.
8-body table 48 B + `puts"O(#{...})"` 14 B = 62 B fixed before the key.

- prefer modulus 16 on the first number: line 1 `N%16` separates all but equal N; the
  second line's first number breaks ties. `%9` on the second keeps the xor range 0..15.
- blunder-episode-3 ruby 98 B, CG 100% (probe `probe/ruby/key98.rb`, 2026-10-24):
  `puts"O(#{%w{n^3 n^2\ log\ n log\ n n\ log\ n n 1 n n n n 2^n n n^2 n}[gets.to_i%16^gets.to_i%9]})"`
- shortest known two-line sum key is `(gets+gets).sum` (15 B) but it includes t and the
  raw value needs a mod; line 1 alone cannot split equal N (needs line 2).
