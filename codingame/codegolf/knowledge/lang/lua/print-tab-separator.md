# print(a,b) separates with TAB

`print(r,4)` writes `r<TAB>4`. Lua 5.4 also rejects `11or` ("malformed number near '11o'"): a numeral needs a space before `or`/`and`.

- local mars-lander-episode-3 referee (split_whitespace) accepts the tab
- CG acceptance for interactive answers: see knowledge/puzzle/mars-lander-episode-3.md (probe 2026-10-21)
- space-safe fallback `print(E.." 4")` costs +5 (`89 ..` needs the space before `..`)
