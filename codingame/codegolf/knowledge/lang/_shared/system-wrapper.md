# system() wrapper

Verified 2026-10-19 on `don't-panic` test 7 (interactive, Python3): `import os;os.system('''bash -c '<core>' ''')` runs `bash` on CG, the child inherits stdin/stdout, interactive turns work, stderr of the child shows in the frame log.

So any lang with a shell call costs wrapper + shortest core in a shell-reachable interpreter (bash; perl/ruby availability unverified). Leaderboard evidence (unverified, arithmetic only): `don't-panic` C top 72 = `main(){system("");}` (19) + bash top 53.

Unverified: which interpreters are on PATH in the CG box besides bash, whether every lang runtime allows process spawn.

Verified on CG 2026-10-20 (shadows-of-the-knight-episode-1, C): `system()` runs `/bin/sh -> dash` (not bash), so a bash core needs `bash -c '...'` (+10 bytes). C wrapper total = `main(){system("bash -c '<core>'");}` = core + 29. Core with `"`, `\` or newline needs C escapes; `;` instead of newline costs nothing. `sed 3q>&2` inside the child skips 3 input lines interactively on CG (1 byte under `head -3>&2`).

Trap (verified 2026-10-20, super-computer): the local referee runs `system()`/`exec`/`os.system` children with local `/bin/sh` = bash, so a core that only works in bash passes locally and fails on CG. Example: `read;` (no variable) in dash prints `read: arg count` and consumes no line. Check every `sh`-run core with dash before submit: `nix build nixpkgs#dash --no-link --print-out-paths`, then `dash -c '<core>' < test.in`, or run the launcher under `bwrap --dev-bind / / --tmpfs /bin --symlink <dash> /bin/sh <launcher>`.
