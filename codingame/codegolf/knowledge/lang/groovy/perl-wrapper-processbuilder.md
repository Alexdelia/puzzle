# Groovy perl wrapper (mayan-calculation, CG accepted 237 #1, 2026-10-21)

`new ProcessBuilder("perl","-ape",'<code>').inheritIO().start()` - perl is in the Groovy CG container. No `sh -c` quoting needed, no `.waitFor()` needed (CG captured the output anyway, -10 B). Perl core: see knowledge/puzzle/mayan-calculation.md.
- native Groovy was 318+: `Eval.xy(x,y,"x${op}y")` avoids Integer overflow (needs Long inputs), BigDecimal from `/` breaks `%`
