Find the shortest algorithm for puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`, "{{TITLE}}"), independent of language.

Scope: `knowledge/puzzle/{{PUZZLE}}.md`, scratch files in your journal directory.

1. Read `puzzle/{{PUZZLE}}/referee/statement.md`, the referee tests and validators, `knowledge/puzzle/{{PUZZLE}}.md`, and `agent/state/leaderboard/{{PUZZLE}}.tsv` (top lengths per language: they bound what is reachable).
2. Search the minimal method: fewest operations, simplest state, math identities, what input can be ignored, what output shortcuts the referee accepts. Prove each idea on the local referee with a short Python or any ready language (`puzzle/{{PUZZLE}}/referee/run.sh <file> --lang <lang> --validators`).
3. Write `knowledge/puzzle/{{PUZZLE}}.md` section "Shortest known": the method, why it is short, which language features it needs (string ops, regex, eval, big ints), lengths you reached, ideas rejected with the reason.
4. Queue the cells that should use the method: `agent/tool/enqueue.sh cell {{PUZZLE}} <lang> --trigger knowledge/puzzle/{{PUZZLE}}.md --depth {{NEXT_DEPTH}} --reason "<method>"`, for every language unless the method is language-specific.

Report `done`.
