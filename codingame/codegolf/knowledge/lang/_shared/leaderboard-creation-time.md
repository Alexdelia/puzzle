# Leaderboard rows carry creationTime

`POST /services/Leaderboards/getFilteredPuzzleLeaderboard` (the service `agent/tool/lib.sh` `leaderboard_rows` uses) returns for each row `creationTime` in milliseconds (epoch) besides `criteriaScore`, `score`, `programmingLanguage`. So before assuming a top is stale ("predates the heavy validators"), convert it and look: a recent date proves the top passes the current validators.

Example (roller-coaster perl, 2026-10-24): Eric 84 @2025-05-24, Wazemo 84 @2026-10-13 — the 84 was current, an older journal's "stale top" theory was wrong.

Query body: `[<leaderboard-id>, null, "global", {active: true, column: "LANGUAGE"|"KEYWORD", filter: <value>}]`; output `.users[]` with `pseudo`, `programmingLanguage`, `criteriaScore`, `score`, `creationTime`. Verified 2026-10-24.

## 2026-10-17 CG validator regime change (old rows kept, not recomputed)

Evidence across puzzles (all fetched 2026-10-25 with `getFilteredPuzzleLeaderboard`, `creationTime` ms):
- ascii-art: old-regime short rows 2026-09-13..10-08; every current-regime row 2026-10-17+; the top 58 was
  re-probed and passes only 1/7 validators today (old-regime artifact).
- the-descent: tops 36 (2024) / 41 (2026-08) vs every 2026-10 row 42+; both outliers predate the change.
- the-fall-episode-3: python 117/117 rows 2026-08-28/29, 122 on 2026-10-17, 136 on 2026-09-27; every
  post-change row is >=859 except our probing-derived 119.

So a displayed 100 % row is not proof of current-validator validity when its `creationTime` falls before the
puzzle's current regime. Do not chase a sub-floor top without checking: (a) creationTime vs the newest rows,
(b) whether every recent (post-change) row is far above it, (c) a probe of the obvious exploit that would
produce it. Old rows keep their stored score; CG does not re-run them.
