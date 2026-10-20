# Hidden-validator exploits

User rule (2026-10-20): hardcoding and exploits are allowed and part of the challenge; on some puzzles finding the hidden exploit is the whole puzzle.

- CG scores a submission on its hidden validators only, not on the IDE tests; tops far below any general method usually hardcode answers keyed on validator inputs or rely on weak validators
- local `validators/` are synthetic (written by our referee tasks): an exploit fails them, which proves nothing about CG
- `agent/tool/submit.sh --probe <file>` skips the local gate and the length check, never marks the referee suspect, prints CG progress + our row, exit 3 when CG did not confirm 100% at that length; cap per task in `agent/tool/config.sh` `probe_submits_per_task`
- CG keeps our best submission per language: a lower score stays hidden behind an existing better row. A probe is informative when it is shorter than our row: the row moves (and commits) iff it scores 100%, so each probe is a 1-bit oracle on a hypothesis about all validators
- test games (`POST /puzzles/<id>/test-games`) only reach IDE tests (validators answer `INVALID_TESTCASE`)
- every fact learned (validator count, value ranges, which input keys separate them, answers) goes in the puzzle's "Hidden validators" section so every language reuses it
- `/progress` score is the best row's score, so a probe's exact score shows only in a language with no row yet (or when it beats the row): spend count probes there, one per language
- validator count from the score grain (verified blunder-episode-3): a constant-output probe (`print"O(1)"`) gives the share of one answer; there 12% plus other scores all multiples of 4 meant 25 validators for 8 IDE tests. Never assume validators = tests
- gate probe: a known-good general method whose output is replaced by garbage when an input feature differs from the IDE test of the same answer; the score counts validators sharing that feature
