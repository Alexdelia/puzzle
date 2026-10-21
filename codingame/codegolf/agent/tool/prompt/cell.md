Golf puzzle `{{PUZZLE}}` (CG `{{PRETTY_ID}}`) in `{{LANG}}` (CG `{{CG_NAME}}`, `.{{EXTENSION}}`).

Scope: `puzzle/{{PUZZLE}}/solution/{{LANG}}/` (one file `main.{{EXTENSION}}`), `knowledge/`.

Leaderboard for this cell (from `agent/state/cell/{{PUZZLE}}/{{LANG}}.json`): {{CELL}}
Goal: length <= top (bytes, `agent/tool/count.sh`), then report `first`.

Our rows of this puzzle in every language (wrapper cores: the shortest perl/python/bash/lua... solution of this puzzle is the core to embed; a first or tied first there is a proven core):

{{PUZZLE_CELLS}}

Our rows of `{{LANG}}` in every puzzle (firsts show the idioms that win in this language):

{{LANG_CELLS}}

Same tables any time: `agent/tool/cells.sh puzzle <puzzle>`, `agent/tool/cells.sh lang <lang>`.

1. Read `knowledge/puzzle/{{PUZZLE}}.md`, `knowledge/lang/{{LANG}}/index.md`, `knowledge/lang/_shared/index.md`, and the trick files that apply.
2. Iterate locally: `puzzle/{{PUZZLE}}/referee/run.sh <file> --validators` (prints `length`). Work on scratch copies; the solution file only ever holds a passing version shorter than our CG row.
3. Each time you beat our row: write `puzzle/{{PUZZLE}}/solution/{{LANG}}/main.{{EXTENSION}}`, run `agent/tool/submit.sh puzzle/{{PUZZLE}}/solution/{{LANG}}/main.{{EXTENSION}}`. A rejection on CG means the local referee missed a rule: note it in the journal and stop submitting this cell.
4. Hidden-validator exploits are allowed and part of the challenge (user rule): hardcoded answers, keying on input features, shortcuts that only CG's hidden validators accept. A top far below any general method means such an exploit exists: finding it is the job, never a reason to plateau. Read the "Hidden validators" section of `knowledge/puzzle/{{PUZZLE}}.md`, `puzzle/{{PUZZLE}}/referee/hidden/facts.md` when present (recovered validators; check against them with `run.sh <file> --hidden`, and when `hidden/complete` exists `submit.sh` checks tests + hidden instead of the synthetic validators) and `knowledge/lang/_shared/hidden-validators.md`. An exploit fails the synthetic local validators, so send it as a probe (at most {{PROBE_CAP}} submits this task, scarce per-puzzle quota). Probes (any submit not meant as the cell solution) live only in `puzzle/{{PUZZLE}}/probe/<lang>/<name>.<ext>` and go through `agent/tool/oracle.sh <probe file> "<question>"` (or `submit.sh --probe <probe file>`); every question and answer is logged in `puzzle/{{PUZZLE}}/probe/probes.tsv`, read it first (`oracle.sh --log {{PUZZLE}}`). A probe CG confirms at 100% and shorter than our row is copied to `solution/<lang>/main.<ext>` and committed as the solution. `solution/<lang>/` only ever holds `main.<ext>`.: CG keeps only our best row per language, so a probe tells something only when it is shorter than our row (row updates and commits iff it scores 100%). Write every validator fact learned into that section.
5. Every new trick you prove (works on the referee, saves bytes, reusable elsewhere): a file in `knowledge/lang/{{LANG}}/` + index line, then queue other cells of `{{LANG}}` (and close languages) that may use it.

Report `first` when length <= top, `continue` if you gained and see more to try, `plateau "<what blocks>"` when out of ideas.
