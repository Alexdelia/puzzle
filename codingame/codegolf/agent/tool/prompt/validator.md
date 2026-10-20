Recover the hidden CG validators of golf puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`, "{{TITLE}}", {{PUZZLE_KIND}}), once for every language.

Scope: `puzzle/{{PUZZLE}}/referee/hidden/`, `knowledge/puzzle/{{PUZZLE}}.md` section "Hidden validators", probe files under `puzzle/{{PUZZLE}}/solution/`, scratch files in your journal directory.

Hardcoding and exploits are allowed and part of the challenge (user rule): the leaders often key on the hidden validators. Recovering them exactly lets every cell hardcode and check locally instead of burning submits.

Known structure:
- validators mirror the IDE tests 1:1: same count, same titles (`agent/tool/cg.sh GET /puzzles/{{PRETTY_ID}}` `testcases`; tests are in `puzzle/{{PUZZLE}}/referee/tests/`). Each validator is a variant of its test: same scenario, other values
- typical variants (user experience): the same map flipped on x or y, mirrored, rotated, translated in x/y, values shifted or renamed, list order permuted, while the answer stays the same or transforms the same way. Generate these candidates from each test first
- local `validators/` are synthetic (ours), not CG's

Sources, cheapest first:
1. Web research (WebSearch, WebFetch): forum thread of the puzzle (`forum.codingame.com`, "validator N fails" posts often quote the input), GitHub repos with CG tests or golf solutions (hardcoded tables reveal keys and answers), code.golf / golf.shinh.org / codegolf.stackexchange. Source URL next to every fact.
2. Probes on the golf leaderboard: `agent/tool/oracle.sh <solution-file> "<question>"` (submits through `submit.sh --probe`, logs the question and answer in `agent/state/oracle/{{PUZZLE}}.tsv`, refuses a question already asked in that language; read past answers first with `agent/tool/oracle.sh --log {{PUZZLE}}`). At most {{PROBE_CAP}} submits this task, agent total 90 per 5 h: plan every probe, ask questions that split the most hypotheses.
   - CG keeps only our best row per language. A probe shorter than our row is a yes/no on "every validator satisfies my predicate" (row moves and commits iff 100%). The first submit in a language where we have no row yet shows its exact score (how many validators pass): a multi-bit answer. Public rows under 100% are allowed (user).
   - typical probe: our correct general solution that answers correctly only when the input matches a hypothesis (candidate transform of test k, a feature in a range, an answer bit), garbage otherwise
   - prefer probes that are themselves a candidate exploit (hardcode keyed on the hypothesized validators): a yes is then also a gain

Artifacts in `puzzle/{{PUZZLE}}/referee/hidden/`:
- `NN.<ext>` (same format and numbering as `tests/`: `NN.in` + `NN.out` for io, `NN.json` for interactive) for every validator recovered exactly; check each with the reference solution: `puzzle/{{PUZZLE}}/referee/run.sh <reference> --lang <lang> --hidden`
- `facts.md`: per validator (number and title): what is known (exact input, transform of test k, features, answer), source (URL, probe line, deduction), confidence; then the shortest key known to separate validators and the answers it maps to
- `complete` (empty file) only when every validator is recovered exactly and the reference passes `--hidden`: `submit.sh` then checks tests + hidden instead of the synthetic validators

Then update `knowledge/puzzle/{{PUZZLE}}.md` section "Hidden validators" (summary + pointer to `facts.md`) and queue the cells that can now use a hardcode: `agent/tool/enqueue.sh cell {{PUZZLE}} <lang> --trigger puzzle/{{PUZZLE}}/referee/hidden/facts.md --depth {{NEXT_DEPTH}} --reason "<what is now known>"`.

Report `done` with what was recovered (even partial: facts.md written), `blocked "<reason>"` only when nothing can be learned without the user.
