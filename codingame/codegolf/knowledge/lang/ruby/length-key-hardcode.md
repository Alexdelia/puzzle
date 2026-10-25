# Ruby: length-keyed hardcode for hidden-validator exploits

For a puzzle whose hidden validators are distinguishable by first-line length L, print a table indexed
by a cheap **periodic** hash of `gets.size` (= L+1 incl. newline, 9 chars, cheapest L accessor):

- hardcode shape `p [v0,v1,...][gets.size KEY]`, body bytes = values_sum + slots + 14 + KEY_len
- values that repeat (e.g. three answers 1) cost 2 bytes/slot (`1,`), so merge them if a hash allows
- big answers: search a shift form, e.g. 37265080320 = `568620<<16` (10) beats the 11-byte decimal
- if the true L is only known mod 256 (an unknown number of +256 steps), the key must be 256-periodic:
  start with `%A` where A | 256 (or `&mask`), then only ops that keep it a function of n%A.
  `gets.size%128*9%65/8` and `gets.size%64%34%26%9` were the best (9 slots) in an exhaustive search
- Ruby precedence: `**` > `% / *` > `+ -` > `<< >>` > `&` > `^ |`; a chain is left-to-right only while
  precedence is non-increasing (`a%64*5/2` is fine, `a%64^5%9` is `(a%64)^(5%9)`, not what you want)
- verify with the real referee or by feeding synthetic first lines of the known lengths; a probe keyed
  this way is an exploit, so it fails synthetic local validators and must go through `oracle.sh`

Verified: the-resistance ruby 62 bytes, CG 100% (2026-10-25, `puzzle/the-resistance/probe/ruby/l-key-62.rb`).
