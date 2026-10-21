# Groovy

- [env](env.md): Groovy 3.0.8 on HotSpot 8u211, script class `Answer` precompiled, local groovyc launcher matches, verified; CG startup ~0.75 s slower than local, 1000 ms first turns flaky
- [syntax-traps](syntax-traps.md): no .9 literal, split() array index throws, print needs space
- [regex-two-pass-chuck-norris](regex-two-pass-chuck-norris.md): Integer.toString(n,2), $1 empty on unmatched, eachLine
- [unbound-var-ternary-init](unbound-var-ternary-init.md): no `s=''` init via ternary else-branch on first line; `=~` as contains, `drop`
- [stdin-chunks-clamp](stdin-chunks-clamp.md): readLines destructure, Matcher getAt, list index bounds, sum() join, verified
- [char-keyed-map-dispatch](char-keyed-map-dispatch.md): letter-keyed direction map, closure-map dispatch, slashy lookup, verified
- [pairs-transpose-median](pairs-transpose-median.md): header+pairs parse via collate(2).transpose(), median deviation sum, verified local
- [perl-wrapper-processbuilder](perl-wrapper-processbuilder.md): ProcessBuilder(perl,-ape,code).inheritIO().start(), no waitFor; perl exists on CG
