Build the referee of puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`, kind `{{PUZZLE_KIND}}`, title "{{TITLE}}").

Scope: `puzzle/{{PUZZLE}}/referee/`, plus `puzzle/{{PUZZLE}}/solution/python/` only through `agent/tool/submit.sh`.

Templates: `puzzle/chuck-norris/referee/` (io) and `puzzle/power-of-thor/referee/` (interactive), library `agent/tool/referee-core` (read its source, use it, do not change it). Spec sections "Referee contract" and "Leaderboard".

1. Statement: `agent/tool/cg.sh GET /puzzles/{{PRETTY_ID}}`, save `.statement` to `puzzle/{{PUZZLE}}/referee/statement.md`.
2. Tests:
   - io: `agent/tool/capture-io.sh {{PUZZLE}}`, then a reference solution and `agent/tool/expect-io.sh {{PUZZLE}} <reference>` (reference must pass every CG test).
   - interactive: echo program that writes its inputs to stderr while playing, run with `cg.sh POST '/puzzles/{{PRETTY_ID}}/test-games?wait=30&format=text'`: the log shows referee state lines and your stderr. Write `tests/<nn>.json` with everything the game needs, including hidden state shown only in referee lines. Port the game rules exactly from the statement and the logs. Probe turn limits (first turn, next turns) with sleeps, 2-3 probes each, never trust the statement value.
3. Referee crate: `Cargo.toml`, `src/main.rs`, `run.sh` as in the templates, clippy and fmt clean.
4. Validators: `validators/` with cases close to what hidden validators may hold (same constraints, edge cases, slightly different values). Expected outputs from the reference.
5. Reference solution: if `puzzle/{{PUZZLE}}/solution/<lang>/` already holds our CG row in a ready lang, it is the reference. Otherwise write a plain correct Python solution at `puzzle/{{PUZZLE}}/solution/python/main.py` and submit it with `agent/tool/submit.sh` once the referee passes it locally: CG confirms 100%.
6. Negative checks: a wrong variant fails, a too-slow variant times out (`run.sh <file> --lang python`).
7. `run.sh <reference> --bench` writes `bench.txt`; gate: referee CPU <= 2 ms per case, suite wall <= 5 s.
8. `knowledge/puzzle/{{PUZZLE}}.md`: rules as referee, probed limits, traps between statement and real behavior, input quirks (final newline, spacing). Template: `knowledge/puzzle/power-of-thor.md`.

Report `done` when all hold. If a rule cannot be reproduced exactly or `referee-core` lacks something, report `blocked "<what and why>"` with the details in the journal.
