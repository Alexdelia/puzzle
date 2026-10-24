# Leaderboard rows carry creationTime

`POST /services/Leaderboards/getFilteredPuzzleLeaderboard` (the service `agent/tool/lib.sh` `leaderboard_rows` uses) returns for each row `creationTime` in milliseconds (epoch) besides `criteriaScore`, `score`, `programmingLanguage`. So before assuming a top is stale ("predates the heavy validators"), convert it and look: a recent date proves the top passes the current validators.

Example (roller-coaster perl, 2026-10-24): Eric 84 @2025-05-24, Wazemo 84 @2026-10-13 — the 84 was current, an older journal's "stale top" theory was wrong.

Query body: `[<leaderboard-id>, null, "global", {active: true, column: "LANGUAGE"|"KEYWORD", filter: <value>}]`; output `.users[]` with `pseudo`, `programmingLanguage`, `criteriaScore`, `score`, `creationTime`. Verified 2026-10-24.
