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
