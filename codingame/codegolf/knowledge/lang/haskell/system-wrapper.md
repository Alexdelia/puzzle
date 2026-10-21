# Haskell system wrapper

`import System.Process;main=system"<shell core>"` (GHC 8.4, `system` returns ExitCode, fine as main). Overhead 35 bytes plus core. super-computer: core `sort -nr|awk '$1+$2>e&&e?0:$2&&e=$1'|wc -l` gives 77, CG-confirmed tie with top. Native Haskell greedy (sort + foldr, `[j|j+d<=e]++a`) is 124. See lang/_shared/system-wrapper.md.

telephone-numbers: core `sed '1d;:a;p;s/.$//;ta'|sort -u|wc -w` gives 72 (CG row #3, top 65). P9 core without `1d` cannot be used here: Haskell cannot post-subtract the `system` output. Native estimates 77+ (unmeasured).
