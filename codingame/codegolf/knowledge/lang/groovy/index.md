# Groovy

- [env](env.md): Groovy 3.0.8 on HotSpot 8u211, script class `Answer` precompiled, local groovyc launcher matches, verified; CG startup ~0.75 s slower than local, 1000 ms first turns flaky (runtime init alone ~1.0-1.3 s, first println of a dynamic expr ~160 ms)
- [syntax-traps](syntax-traps.md): no .9 literal, split() array index throws, print needs space
- [regex-two-pass-chuck-norris](regex-two-pass-chuck-norris.md): Integer.toString(n,2), $1 empty on unmatched, eachLine
- [unbound-var-ternary-init](unbound-var-ternary-init.md): no `s=''` init via ternary else-branch on first line; `=~` as contains, `drop`
- [stdin-chunks-clamp](stdin-chunks-clamp.md): readLines destructure, Matcher getAt, list index bounds, sum() join, verified
- [char-keyed-map-dispatch](char-keyed-map-dispatch.md): letter-keyed direction map, closure-map dispatch, slashy lookup, verified
- [pairs-transpose-median](pairs-transpose-median.md): header+pairs parse via collate(2).transpose(), median deviation sum, verified local
- [perl-wrapper-processbuilder](perl-wrapper-processbuilder.md): ProcessBuilder(perl,-ape,code).inheritIO().start(), no waitFor; perl exists on CG
- [interactive-first-turn](interactive-first-turn.md): interactive frame-1 deadline ~1.5 s JVM uptime on CG, main->first answer <= ~90 ms local or timeout, local referee does not model it, verified; the-descent `(0..15).max{System.in.read()}` 48 also flaky (80%)
- [prefix-set-take-sum](prefix-set-take-sum.md): `(1..20).sum{x*.take(it)}.toSet()` distinct prefixes; char-list inits form OOMs on CG
- [perl-wrapper-escape-d](perl-wrapper-escape-d.md): `\d` must be `\\d` in single-quoted Groovy strings; interactive perl core wrapper, 287
- [io-time-limit](io-time-limit.md): CG io deadline ~1.93 s JVM uptime on every io puzzle (python/bash/js 6 s), wall not CPU, native parse of 1 MB times out, use sh/perl wrapper, local referees blind except the-resistance (1100 ms), verified
- [python-wrapper-processbuilder](python-wrapper-processbuilder.md): /usr/bin/python3 exists on CG, slashy-string wrapper, local dev shell lacks python3 (probe only)
- [byte-read-max-key](byte-read-max-key.md): `(0..15).max{System.in.read()}>>1` argmax per turn, key called once per element, 48 bytes, CG 79.976 (timing?)
