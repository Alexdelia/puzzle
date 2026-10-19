# system() wrapper

Verified 2026-10-19 on `don't-panic` test 7 (interactive, Python3): `import os;os.system('''bash -c '<core>' ''')` runs `bash` on CG, the child inherits stdin/stdout, interactive turns work, stderr of the child shows in the frame log.

So any lang with a shell call costs wrapper + shortest core in a shell-reachable interpreter (bash; perl/ruby availability unverified). Leaderboard evidence (unverified, arithmetic only): `don't-panic` C top 72 = `main(){system("");}` (19) + bash top 53.

Unverified: which interpreters are on PATH in the CG box besides bash, whether every lang runtime allows process spawn.
