Find the shortest algorithm for puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`, "{{TITLE}}"), independent of language.

Scope: `knowledge/puzzle/{{PUZZLE}}.md`, `knowledge/lang/_shared/`, probe files in `puzzle/{{PUZZLE}}/probe/`, scratch files in your journal directory.

1. Read `puzzle/{{PUZZLE}}/referee/statement.md`, the referee tests and validators, `knowledge/puzzle/{{PUZZLE}}.md`, and `agent/state/leaderboard/{{PUZZLE}}.tsv` (top lengths per language: they bound what is reachable).
2. Search the minimal method: fewest operations, simplest state, math identities, what input can be ignored, what output shortcuts the referee accepts. Prove each idea on the local referee with a short Python or any ready language (`puzzle/{{PUZZLE}}/referee/run.sh <file> --lang <lang> --validators`).
3. Hidden-validator exploits are allowed and part of the challenge (user rule): hardcoded answers, keying on input features, shortcuts that only CG's hidden validators accept. When tops sit below any general method, find the exploit: what the hidden validators likely hold (same generator as the tests, small variations), which features separate them, the shortest key. Probe CG with `agent/tool/oracle.sh puzzle/{{PUZZLE}}/probe/<lang>/<name>.<ext> "<question>"` (at most {{PROBE_CAP}} submits this task, scarce per-puzzle quota; each submit prints CG's pass/fail for every hidden validator, whatever our row; without that report, a probe tells something only when shorter than our row in that language, since CG keeps the best row). See `knowledge/lang/_shared/hidden-validators.md`. Write section "Hidden validators" in `knowledge/puzzle/{{PUZZLE}}.md`: facts proven, guesses, probes spent and their results.
4. Write `knowledge/puzzle/{{PUZZLE}}.md` section "Shortest known": the method, why it is short, which language features it needs (string ops, regex, eval, big ints), lengths you reached, ideas rejected with the reason.
5. Queue the cells that should use the method: `agent/tool/enqueue.sh cell {{PUZZLE}} <lang> --trigger knowledge/puzzle/{{PUZZLE}}.md --depth {{NEXT_DEPTH}} --reason "<method>"`, for every language unless the method is language-specific.

Report `done`.
