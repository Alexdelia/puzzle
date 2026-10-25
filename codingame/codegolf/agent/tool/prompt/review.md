Final review of every `{{LANG}}` (CG `{{CG_NAME}}`, `.{{EXTENSION}}`) golf solution: shave the bytes the cell workers left, firsts and ties included. A tie or a 1-byte lead is fragile: every byte under the top keeps rank 1 when the others improve.

Scope: `puzzle/*/solution/{{LANG}}/main.{{EXTENSION}}`, `knowledge/`.

Our solutions (`agent/tool/cells.sh lang {{LANG}}`; `agent/tool/cells.sh puzzle <p>` shows every language of a puzzle, to find the shortest wrapper core):

{{LANG_CELLS}}

Earlier journals list the solutions already reviewed with the commit they were reviewed at: skip a solution whose `git log -1 --format=%h -- <file>` is unchanged since, review the rest. Order: ties and 1-2 byte leads first, then the other firsts, then the rest by gap.

Per solution, cheapest first:
1. mechanical: whitespace and delimiters the language does not need (Clojure: `["perl" "-e" "..."]` -> `["perl""-e""..."]`, `(ProcessBuilder. [` -> `(ProcessBuilder.[`), redundant parens, quotes, separators, trailing newline, a longer spelling of the same call chain
2. consistency: a shorter idiom used in another `{{LANG}}` solution or listed in `knowledge/lang/{{LANG}}/index.md` that this one does not use yet (main declaration, input reading, output, wrapper form)
3. wrapper core: when the solution spawns another interpreter (perl, python3, bash, sh, lua), compare its embedded core with that interpreter's own solution of the same puzzle (`puzzle/<p>/solution/<core lang>/`) and with the cores in the other wrappers of the puzzle; take the shortest that survives the quoting
4. a few minutes of real golfing on the ties and smallest leads

Each gain: work on a scratch copy, check it with `puzzle/<p>/referee/run.sh <file> --cg` (only `hidden/` when `puzzle/<p>/referee/hidden/complete` or `hidden/keyed` exists, as CG grades), copy it to `puzzle/<p>/solution/{{LANG}}/main.{{EXTENSION}}` only when it passes, then `agent/tool/submit.sh puzzle/<p>/solution/{{LANG}}/main.{{EXTENSION}}`. Batch every gain of one puzzle into one submit. A CG rejection means the local referee missed a rule: write it in the journal, put the committed version back (`git show HEAD:./<path> > <path>` from `codegolf/`), move on. No probes: an idea that needs one, or more work than a polish, goes in the journal and in `knowledge/puzzle/<p>.md`; when the cell is not first, also queue it: `agent/tool/enqueue.sh cell <p> {{LANG}} --trigger <knowledge file> --depth {{NEXT_DEPTH}} --reason "<idea>"`.

Journal one line per reviewed solution: `<puzzle> <commit reviewed> <length before> -> <length after> <what changed | nothing>`. A trick that saved bytes and applies elsewhere: a file in `knowledge/lang/{{LANG}}/` + index line.

Report `continue` when time or budget runs short with solutions left (the next review resumes from your journal), `done` when every solution is reviewed.
