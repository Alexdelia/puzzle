# Substring test without Str

Verified locally and on CG (genome-sequencing row 254, 2026-10-22).

- no `Str` on CG, no stdlib substring search in 4.14: `let rec m t u i=try sub u i(length t)=t||m t u(i+1)with _->false` inside `String.(...)`, call `m t u 0` = "t in u"; `sub` out of range raises `Invalid_argument`, the `try` turns it into `false`
- empty `t` gives `true` at once (useful as an initial state)
- shrinking `u` instead of an index (`m t(sub u 1(length u-1))`) is 12 longer net
- several recursive functions: one `let rec m..and g..and f..` chain, each `and` saves 4 over `let rec`
