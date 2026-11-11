# contest

Claude Code plugin that plays a CodinGame contest alone. A headless supervisor (Rust, `runtime/`) runs a manager tick and sandboxed workers, each a fresh `claude -p`, plus non-LLM jobs under a core/RAM broker. The user talks to the run through the `/contest` skill or the `contest` CLI.

## Build

Load: `claude --plugin-dir <path-to>/plugin` (puts `bin/` on the Bash PATH, adds the `/contest` skill). `bin/contest` builds `runtime/target/release/contest` on first use. Rebuild by hand: `cargo build --release` in `runtime/`. Needs bwrap; pasta comes from PATH or `nix build nixpkgs#passt`.

## Use

- init: `contest init <dir> --puzzle <id> --mode contest-live|past-puzzle|optimization [--deadline <rfc3339>] [--active-budget-hours N]`; `<dir>` inside a git repo, not its top
- start or resume: `contest start <dir>` (forks the supervisor, returns)
- tell it something: `contest say <dir> "<text>"`; the manager replies in `run/replies.md`
- status: `contest status <dir>`
- stop: `contest stop <dir> [--reason <text>] [--wait]`; workers get 270 s to save notes and commit, then SIGKILL

In Claude Code: `/contest [dir] status|say|start|stop`.

## State

All under `<dir>/run/`:

- `config.toml`: workers, models, budgets, limits (edit before start)
- `run.json`: puzzle, mode, phase, active time
- `status.md`: summary, <= 40 lines; `replies.md`: manager replies
- `journal.md`: one line per decision; `events.jsonl`, `ledger.tsv` (tokens, cost per agent run)
- `tasks/<id>/`: `meta/` (spec, state), `notes.md` (handoff), `result.md`
- `jobs/<id>/`: record, lease, log
- `inbox/`, `STOP`, `supervisor.log`, `agents/` (stream logs, sandbox homes)

Workers commit only through `contest commit` (subject `<kind>: <task-id> <milestone>`); nothing pushes.

## Smoke test

`runtime/smoke.sh <empty-dir>`: real haiku run in a temp git repo (hello task commit, then stop during a job). Expects `smoke: ok`, cost under 1 USD.
