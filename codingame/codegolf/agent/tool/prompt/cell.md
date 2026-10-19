Golf puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`) in `{{LANG}}` (CG `{{CG_NAME}}`, `.{{EXTENSION}}`).

Scope: `puzzle/{{PUZZLE}}/solution/{{LANG}}/` (one file `main.{{EXTENSION}}`), `knowledge/`.

Leaderboard for this cell (from `agent/state/cell/{{PUZZLE}}/{{LANG}}.json`): {{CELL}}
Goal: length <= top (bytes, `agent/tool/count.sh`), then report `first`.

1. Read `knowledge/puzzle/{{PUZZLE}}.md`, `knowledge/lang/{{LANG}}/index.md`, `knowledge/lang/_shared/index.md`, and the trick files that apply.
2. Iterate locally: `puzzle/{{PUZZLE}}/referee/run.sh <file> --validators` (prints `length`). Work on scratch copies; the solution file only ever holds a passing version shorter than our CG row.
3. Each time you beat our row: write `puzzle/{{PUZZLE}}/solution/{{LANG}}/main.{{EXTENSION}}`, run `agent/tool/submit.sh puzzle/{{PUZZLE}}/solution/{{LANG}}/main.{{EXTENSION}}`. A rejection on CG means the local referee missed a rule: note it in the journal and stop submitting this cell.
4. Every new trick you prove (works on the referee, saves bytes, reusable elsewhere): a file in `knowledge/lang/{{LANG}}/` + index line, then queue other cells of `{{LANG}}` (and close languages) that may use it.

Report `first` when length <= top, `continue` if you gained and see more to try, `plateau "<what blocks>"` when out of ideas.
