# Self re-exec loop via `./*` glob (compiled langs)

Verified 2026-10-22 on CG, OCaml (the-descent 52, 100%, rank 1, top was 53): a `system()` child can loop by re-running the program binary at the end of the shell command: `Sys.command"<one-turn pipeline>;./*"`.

- CG OCaml box (test game 910915820, `env;pwd;ls -l /tmp`): cwd = `/tmp`, `HOME=/tmp`, binary `/tmp/Answer` (pid 1), next to `Answer.cmi Answer.cmx Answer.ml Answer.o`, a `deps/` dir and stray `*.class` files of other runs (`ChaînonParcours.class`, `Donnée.class`, `Player$ActionBâbord.class`, `Unité.class`)
- dash expands `./*` sorted, `./Answer` comes first (A < C D P U d), the other names become ignored arguments: 3 B vs `/tmp/Answer` 11 B, `./A*` 4 B is the safer fallback if a file sorting before `Answer` ever appears
- no `_` env var on CG: dash `$_` is empty there, not the binary path
- env also has `LD_PRELOAD=libstdbuf.so` `_STDBUF_O=0`: coreutils tools write unbuffered
- each turn nests one more sh + binary (depth = turns, fine for ~10-turn games); the game end kills the chain on CG
- TRAP local: the referee cwd is the codegolf root, so `./*` fails locally: submit as a probe. Run by hand at EOF the loop never stops and grows an unbounded process chain; `pkill -f <pattern>` also kills your own shell, kill by `ps -eo pid,comm | awk '$2=="<binary>"'`
- other compiled boxes (C, C++, D, Pascal, Rust, Go, Swift...): cwd and binary name unverified, dump `pwd;ls` in one test game first
