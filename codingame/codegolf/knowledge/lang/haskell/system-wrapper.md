# Haskell system wrapper

`import System.Process;main=system"<shell core>"` (GHC 8.4, `system` returns ExitCode, fine as main). Overhead 35 bytes plus core. super-computer: core `sort -nr|awk '$1+$2>e&&e?0:$2&&e=$1'|wc -l` gives 77, CG-confirmed tie with top. Native Haskell greedy (sort + foldr, `[j|j+d<=e]++a`) is 124. See lang/_shared/system-wrapper.md.

telephone-numbers: core `sed '1d;:a;p;s/.$//;ta'|sort -u|wc -w` gives 72 (CG row #3, top 65). P9 core without `1d` cannot be used here: Haskell cannot post-subtract the `system` output. Native estimates 77+ (unmeasured).

## Interpreters in the CG Haskell box (verified 2026-10-22)

- `command -v` probe (telephone-numbers-codesize test 1, Haskell): `/usr/bin/python3` (3.11.5), `/usr/bin/python` (version not probed), `/bin/lua` (5.4.4), `/bin/php` (7.3.9 cli), `/usr/bin/perl`, `/bin/bash`; NO `ruby`, NO `node`/`nodejs`
- `system"python3 -c'..'"`, `system"lua -e'..'"`, `system"php -r'..'"` all pass temperature-code-golf test 4 on CG (journal `wrap.-.haskell.1792694203180122113`)
- `import System.Cmd;main=rawSystem"lua"["-e","<core>"]` also passes on CG: no shell, so a core with `'` needs no `'\''` (3 bytes each); 7 bytes over `system"lua -e'<core>'"`, wins from 3 quotes up
- `system` waits for the child, so interactive puzzles need nothing more (no Player/Solution entry point in Haskell)
- local launcher has no `php`: Haskell php wrappers fail locally, pass on CG
- generators in `agent/wrap/haskell/` emit these forms for every core
