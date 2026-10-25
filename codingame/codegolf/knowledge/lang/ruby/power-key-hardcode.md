# Ruby: `**` exponent keys widen the perfect-hash space of a hardcode

Verified 2026-10-25 on the-resistance ruby: `p [1,8,0,568620<<16,2,64,2**30,150569][gets.size**8/62%9]`
is CG 100 % at **57 bytes** (tie #1; probe 41563668). The previous exhaustive searches of
`gets.size`+chain keys (keys3/keys4/cov2) had no `**`, so they missed every exponent key.

- Shape: `p [TABLE][gets.size**d REST]`; total = vs + m + len(K) + 5
  (vs = sum of value chars, m = array length = number of slots, K = key expression).
  8 distinct answers cost vs=27; an 8-slot layout (m=8) leaves 17 chars for K at 57 B.
- Ruby precedence: `**` binds tighter than unary minus and `* / %`, so `gets.size**8/62%9`
  is `((gets.size**8)/62)%9`; `gets.size**8%9` is `(gets.size**8)%9`. `**` is right-assoc.
- Exponents d=2..72 suffice for the-resistance (n<100000): the final `%M` sees `n**d mod M`,
  whose period in d divides lambda(M); big ints are exact in Ruby, but in C use `__int128` up to
  d~7 or modular exponentiation. `n**d` can be huge; compute/verify in Python or Ruby, never with
  a plain int64 in C (and never parse `**` left-to-right as `*0*`).
- A `**` key is still an exact-stat hardcode: it passes only the inputs whose stat lands in the
  64 slot. Verify the candidate on every covered value of the varying stat, then send it as a probe.
- Search recipe that found 57: enumerate `**d` (d=2..72) + a chain suffix of <= (budget) chars,
  keep layouts consistent on the 9 fixed validators, compute coverage of the candidate k set,
  pick the key covering the actual k. The 57-byte key covers 50/389 k of L9; the hit pinned k to it.
