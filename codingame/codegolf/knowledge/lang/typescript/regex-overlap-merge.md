# Regex overlap merge (genome-sequencing, 124 B, CG 100% 2026-10-21)

- `s=t.match(s)?t:s.match(t)?s:(s+`<LF>`+t).replace(/(.*)\n\1/,'$1')` as for-update: merges `t` into `s` with max suffix/prefix overlap (leftmost `(.*)\n\1` = longest), real newline inside the template (1 B)
- beats the k-loop fold (134) by 10 B; `var s='',t` in the for head needed (tsc rejects undeclared), `console.log` (print undeclared)
- puzzle fact: `t.match(s)?t` merges the reset (`s` in `t`) into the same ternary
