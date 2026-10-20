# system() wrapper

Verified 2026-10-19 on `don't-panic` test 7 (interactive, Python3): `import os;os.system('''bash -c '<core>' ''')` runs `bash` on CG, the child inherits stdin/stdout, interactive turns work, stderr of the child shows in the frame log.

So any lang with a shell call costs wrapper + shortest core in a shell-reachable interpreter (bash; perl/ruby availability unverified). Leaderboard evidence (unverified, arithmetic only): `don't-panic` C top 72 = `main(){system("");}` (19) + bash top 53.

Unverified: which interpreters are on PATH in the CG box besides bash, whether every lang runtime allows process spawn.

Verified on CG 2026-10-20 (shadows-of-the-knight-episode-1, C): `system()` runs `/bin/sh -> dash` (not bash), so a bash core needs `bash -c '...'` (+10 bytes). C wrapper total = `main(){system("bash -c '<core>'");}` = core + 29. Core with `"`, `\` or newline needs C escapes; `;` instead of newline costs nothing. `sed 3q>&2` inside the child skips 3 input lines interactively on CG (1 byte under `head -3>&2`).

Trap (verified 2026-10-20, super-computer): CG `system()`/`exec`/`os.system`/backticks children run `/bin/sh` = dash, so a core that only works in bash fails on CG. Example: `read;` (no variable) in dash prints `read: arg count` and consumes no line; `read a;` works. Since 2026-10-20 every local launcher runs under bwrap with `/bin/sh` -> dash 0.5.11 (`agent/tool/lang/cache.sh`), so the local referee catches it; outside the referee check with `$CG_DASH/bin/dash -c '<core>' < test.in`.

Verified 2026-10-20 (mars-lander-episode-3, Bash test game 910407501): the CG Bash box has `/usr/bin/python3` (3.11.5), `/usr/bin/perl`, `/bin/php` on PATH (no node). Perl `exec"/usr/bin/python3",...` and Ruby `exec"/usr/bin/python3","-c",DATA.read` also run Python on CG (submits 100%/50% as predicted). Local perl/ruby sandboxes have empty PATH and no python3: such wrappers fail locally only.

Verified 2026-10-20 (blunder-episode-1, non-zero partial scores): `/usr/bin/perl` with inherited stdin is reachable from Ruby `exec"/usr/bin/perl","-e",DATA.read` + `__END__`, PHP `passthru("/usr/bin/perl -e ".escapeshellarg(<<<'P' ... P))`, JavaScript `require("child_process").execFileSync("/usr/bin/perl",["-e",String.raw`...`],{stdio:"inherit"})`, Lua `os.execute("/usr/bin/perl -e '"..[==[...]==].."'")`. Handy to run one perl probe core in many fresh languages.
