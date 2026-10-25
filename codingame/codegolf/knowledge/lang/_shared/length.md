# Length

CG golf length = UTF-8 bytes of the code, trailing newlines excluded. Local: `agent/tool/count.sh`.

- probe 2026-10-19, Ruby on `temperature-code-golf`: code with `é€😀😀` and final `\n`: 75 bytes, 66 code points, 68 UTF-16 units, CG length 74
- 13 older solutions (Python, C, C++, Bash) all match `bytes - 1` with a final `\n`
- consequence: packing ASCII into multi-byte chars (`exec(bytes('...','u16')[2:])`) costs bytes, never saves
- final newline free: keep it or not, same length

## Byte counting started 2023-03-29, old rows were recomputed (authoritative)

CG staff announced the switch: "Starting from today, the code size in these two formats will
take into account the number of bytes, instead of the number of characters ... the code golf
puzzles ... scores and leaderboards will be recomputed by tomorrow"
(forum.codingame.com/t/char-vs-bytes-clash-of-code-shortest-and-code-golf-puzzles/200149,
_CG_ClementHammel, 2023-03-29). Consequences:

- a leaderboard row's `criteriaScore` is a **byte** count regardless of `creationTime`; rows
  created before 2023-03-30 were rescaled at the switch
- the earlier "stale char-counted row" heuristic (a top below the recent cluster is
  pre-byte-switch/unreachable) is **wrong** for any row created after 2023-03: such a top is
  a genuine byte golf solution. `creationTime` only matters for rows older than 2023-04.
  Example: the-resistance ruby top 57 (2025-04-18 and 2026-09) is real; temperature perl 29
  (2023-07..2026-09 rows) is real
- Ruby char-era packing overhead was ~22-25 chars (`eval(...).encode'UTF-16BE'.b`), so packing
  was worthwhile for payloads > ~55 chars then; it is useless now (costs bytes)
