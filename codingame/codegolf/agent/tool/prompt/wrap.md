Write the wrapper generators of host language `{{LANG}}` (CG `{{CG_NAME}}`, `.{{EXTENSION}}`). `agent/tool/wrap.sh <puzzle>` feeds them every core solution of a puzzle (our perl, python, bash... solutions), keeps per host the shortest candidate that passes the local referee and submits it, with no agent; `agent/tool/wrapd.sh` reruns it after every core gain. Your generators make every core gain reach `{{LANG}}` for free, and give `{{LANG}}` a row in every puzzle where it has none.

Scope: `agent/wrap/{{LANG}}/`, `knowledge/`. Existing files there: extend them (new cores, shorter forms, `unknown` availability to settle), do not start over.

Contract (read `agent/wrap/lib.sh` and the two references `agent/wrap/clojure/`, `agent/wrap/c/` first):
- `agent/wrap/{{LANG}}/<core>.sh <core-file> <io|interactive> <out-dir>`, executable, one per core lang slug (perl python bash ruby lua php javascript): sources `../lib.sh` (`$core_body`, `$core_flags` from a perl shebang, `perl_args`, the escapers, `shell_command`, `emit <ext> <text>`) and emits one or more candidates. Emit several when the shortest form depends on the core (quote style, `sh` vs `bash`, escaping variant): `wrap.sh` keeps the shortest that passes, so a variant that breaks on some cores costs nothing
- io and interactive differ: interactive needs the child on the same stdin/stdout and the main waiting for it (CG kills the children at main exit: `knowledge/lang/_shared/orphan-child-kill.md`), and the entry point CG expects on interactive puzzles (Player vs Solution, see `knowledge/lang/{{LANG}}/`)
- helpers only `{{LANG}}` needs: `agent/wrap/{{LANG}}/common.sh` (never a lang slug: `<slug>.sh` means a generator), sourced, not executable. `agent/wrap/lib.sh` is shared and the user's: a change there is a proposal in your journal
- `agent/wrap/{{LANG}}/available.tsv`: header `core<TAB>status<TAB>evidence`, status `yes|no|unknown`; `wrap.sh` only uses `yes`. Evidence: a CG test game id, or a committed CG-accepted solution that runs that interpreter from `{{LANG}}`

Steps:
1. Know how `{{LANG}}` spawns a process with inherited stdin/stdout and waits for it: `knowledge/lang/{{LANG}}/`, `knowledge/lang/_shared/system-wrapper.md`, our existing `{{LANG}}` wrapper solutions (`agent/tool/cells.sh lang {{LANG}}`, files that run perl/python/bash). Pick the shortest form per core.
2. Write the generators, then check them locally: `agent/tool/wrap.sh <puzzle> --host {{LANG}}` (check mode: prints our length and the shortest passing candidate, submits nothing) on at least one io and one interactive puzzle, plus `--estimate` on all puzzles to catch scripts that fail. The local launcher has perl, python3 and lua on PATH: a local pass does not prove CG has them.
3. Availability: for each core, one CG test game of a generated candidate on an easy IDE test (`agent/tool/cg.sh POST /puzzles/<pretty_id>/test-games`, see `.doc/spec.md`); passing = `yes` with the game id, "not found" style errors = `no`. Skip a core already proven by a committed CG-accepted solution. About 8 test games for this task.
4. Facts worth sharing (an interpreter missing from the box, a quoting trap): `knowledge/lang/{{LANG}}/` + index line, or `knowledge/lang/_shared/system-wrapper.md`.

Do not submit: `wrapd.sh` does it once this task reports `done` (`finish.sh` commits `agent/wrap/{{LANG}}/` and queues every puzzle).

Report `done` when at least one core is `yes` with a working generator, `blocked "<reason>"` when `{{LANG}}` cannot spawn a process on CG.
