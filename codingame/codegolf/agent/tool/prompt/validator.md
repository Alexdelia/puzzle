Recover the hidden CG validators of golf puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`, "{{TITLE}}", {{PUZZLE_KIND}}), once for every language.

Scope: `puzzle/{{PUZZLE}}/referee/hidden/`, `knowledge/puzzle/{{PUZZLE}}.md` section "Hidden validators", probe files in `puzzle/{{PUZZLE}}/probe/`, scratch files in your journal directory.

Hardcoding and exploits are allowed and part of the challenge (user rule): the leaders often key on the hidden validators. Recovering them exactly lets every cell hardcode and check locally instead of burning submits.

Known structure:
- validators often mirror the IDE tests 1:1, same count and titles (mayan-calculation: 12 and 12), but not always (the-resistance: 10 validators for 6 tests): count them first (the report of any submit lists every validator by number and title). IDE tests: `agent/tool/cg.sh GET /puzzles/{{PRETTY_ID}}` `testcases`, files in `puzzle/{{PUZZLE}}/referee/tests/`. When 1:1, each validator is a variant of its test: same scenario, other values
- typical variants (user experience): the same map flipped on x or y, mirrored, rotated, translated in x/y, values shifted or renamed, list order permuted, while the answer stays the same or transforms the same way. Generate these candidates from each test first
- local `validators/` are synthetic (ours), not CG's

Sources, cheapest first:
1. Web research (WebSearch, WebFetch): forum thread of the puzzle (`forum.codingame.com`, "validator N fails" posts often quote the input), GitHub repos with CG tests or golf solutions (hardcoded tables reveal keys and answers), code.golf / golf.shinh.org / codegolf.stackexchange. Source URL next to every fact.
2. Probes on the golf leaderboard: `agent/tool/oracle.sh puzzle/{{PUZZLE}}/probe/<lang>/<name>.<ext> "<question>"` (submits through `submit.sh --probe`, logs the question and answer in `puzzle/{{PUZZLE}}/probe/probes.tsv`, committed with the probe sources, refuses a question already asked in that language; read past answers first with `agent/tool/oracle.sh --log {{PUZZLE}}`). At most {{PROBE_CAP}} submits this task; CG counts submits per puzzle (5 / 5 min, 20 / 30 min, 40 / 1 h, 100 / 5 h), agents stop at 90 per puzzle per 5 h, cells of this puzzle share them: plan every probe, ask questions that split the most hypotheses.
   - every submit prints CG's report of that submission: pass/fail of each hidden validator by number and title, and its weight (`submit.sh: probe result, report ...`, logged in `probes.tsv`, raw JSON in `agent/state/report/<submission-id>.json`, read again with `agent/tool/cg-report.sh <submission-id>`). It holds whatever our row: a probe longer than our row or under 100% still answers per validator, so one probe is one bit per validator. A predicate probe (correct answer iff the input satisfies P, garbage otherwise) tells exactly which validators satisfy P: bisect value ranges, test candidate transforms of test k on validator k, read answer bits
   - when `submit.sh` prints `report unavailable`, fall back to the row oracle (`knowledge/lang/_shared/hidden-validators.md`, "Row oracle"): CG keeps only our best row per language, a probe shorter than our row is a yes/no on "every validator satisfies my predicate", the first submit in a language with no row shows its exact score. Public rows under 100% are allowed (user).
   - typical probe: our correct general solution that answers correctly only when the input matches a hypothesis (candidate transform of test k, a feature in a range, an answer bit), garbage otherwise
   - prefer probes that are themselves a candidate exploit (hardcode keyed on the hypothesized validators): a yes moves our row (`probe gain`, solution file untouched): record the exploit in `facts.md` for the cell task

Artifacts in `puzzle/{{PUZZLE}}/referee/hidden/`:
- `NN.<ext>` (same format and numbering as `tests/`: `NN.in` + `NN.out` for io, `NN.json` for interactive) for every validator recovered exactly; check each with the reference solution: `puzzle/{{PUZZLE}}/referee/run.sh <reference> --lang <lang> --hidden`
- `facts.md`: per validator (number and title): what is known (exact input, transform of test k, features, answer), source (URL, probe line, deduction), confidence; then the shortest key known to separate validators and the answers it maps to
- `complete` (empty file) only when every validator is recovered exactly and the reference passes `--hidden`: `run.sh --cg`, `submit.sh` and `wrap.sh` then check only `hidden/` (what CG grades), not the IDE tests nor the synthetic validators
- `keyed` (empty file, written by referee tasks): `hidden/NN.*` are reconstructions, exact only in what `facts.md` pins (class key, answer), the rest generated; same `--cg` behavior as `complete`. Overwrite them with exact inputs as you recover them; once every validator is exact, delete `keyed`, write `complete` and remove the "Referee reconstruction" section of `facts.md`

Then update `knowledge/puzzle/{{PUZZLE}}.md` section "Hidden validators" (summary + pointer to `facts.md`) and queue the cells that can now use a hardcode: `agent/tool/enqueue.sh cell {{PUZZLE}} <lang> --trigger puzzle/{{PUZZLE}}/referee/hidden/facts.md --depth {{NEXT_DEPTH}} --reason "<what is now known>"`.

Report `done` with what was recovered (even partial: facts.md written), `blocked "<reason>"` only when nothing can be learned without the user.
