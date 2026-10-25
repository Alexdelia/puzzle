# CG test-game sandbox: PID 1, cwd /tmp, no per-testcase argv/env

Verified 2026-10-25 on `power-of-thor` via `POST /api/v1/puzzles/power-of-thor/test-games`
(`{"language":"Ruby","code":...,"testcase":1}`; interactive puzzle: response has `log` with
`stderr:` lines for our own program only):

- the submitted program runs as **PID 1** in a fresh PID namespace (`warn $$` logged `1`)
- `Dir.pwd` is `/tmp`, the script is `/tmp/Answer.rb`, `ARGV` is empty
- `ENV` has no testcase/validator id: LC_ALL, PATH, PYTHONHOME/PYTHONPATH, HOME, TEMP,
  LD_LIBRARY_PATH, GNUSTEP_*, GROOVY_HOME, `_STDBUF_O/E=0`, `LD_PRELOAD=libstdbuf.so`
- our own stderr is visible in the test-game log (`stderr:` under the frame); stdout is shown
  per frame too. Useful to read process/environment facts without spending a submission.
- whitespace does not split moves: `puts "W "*21` on testcase 1 -> referee
  `Expected a movement (...) but found 'W W W ... '` and lose; the whole line is the movement.

Implications: a `$$`-keyed random/hardcode cannot distinguish the hidden validators **if each
validator run gets its own fresh sandbox** (PID 1 everywhere) - the same plan is emitted for all
four. The local referee's launcher, by contrast, runs the 4 cases with PIDs +3 apart (ruby
7243/7246/7249/7252, perl 1599/1602/1605/1608), which is *not* evidence about CG's runner.

## Perl confirmation + strict move parsing + sandbox contents (2026-10-25, power-of-thor)

Same facts measured with `{language:"Perl",code,testcase:1}` test games (log via the
response; only ONE stderr line per frame is shown, put all diagnostics on one line):

- Perl solution also runs as **PID 1** (`warn` shows as the frame `error:` line),
  `/tmp/Answer.pl`, `$^X=/usr/bin/perl`, `$]=5.034000`, `$^O=linux`, cwd `/tmp`, ARGV empty,
  env exactly: GNUSTEP_HOST, GNUSTEP_HOST_CPU, GNUSTEP_HOST_OS, GNUSTEP_IS_FLATTENED,
  GROOVY_HOME, HOME, LC_ALL, LD_LIBRARY_PATH, LD_PRELOAD, PATH, PYTHONHOME, PYTHONPATH,
  TEMP, _STDBUF_E, _STDBUF_O
- sandbox fs: `/tmp` = `Answer.pl` + empty `deps`; `/codemachine/lib/<lang>` (incl. ruby),
  `/data/mnist`, `/taskout` empty, `/home/cartser` empty. No testcase id in any file/env/arg.
- **moves are strict**: `X`, ` W ` (spaces), `w` (lowercase) and an empty line all lose at
  once with `Expected a movement (one of N, NE, E, S, SE, SW, W, NW) but found '...'`.
  No trimming, no case folding, no invalid-move no-op.
=> keying on `$$`/`$0`/env/argv cannot distinguish validators if they share this sandbox;
the only key is the game input.

## Bash runner + persistent /tmp (2026-10-25, power-of-thor bash test games 911770328/389/619)

- Bash submission also runs as **PID 1** (`/proc/1/cmdline` = `/bin/bash /tmp/Answer.sh`, PPID 0),
  cwd `/tmp`, `HOME=/tmp`, no testcase/validator id in env or argv; on CG `/tmp/Tester.sh` is
  **absent** now (older 2026-10-19 chuck-norris capture saw a parent `bash -c ... source
  /tmp/Tester.sh`: the runner was changed since).
- `/tmp` is a **persistent shared volume, not a fresh per-test dir**: it held other users'
  stale `.class` files from Jul-Oct 2026 next to `Answer.sh`. So a state file survives between
  sandbox runs on the same host; whether the four validator containers share it is unverified.
- directive redirection trap when dumping diagnostics: `cmd 2>&1 >&2` sends stderr to the
  game's *stdout* (the referee reads it as a move and loses at frame 1); use `{ ...; } >&2`.
- lowercase moves are rejected in Bash too: `yes w` on testcase 1 -> `found 'w'`.
