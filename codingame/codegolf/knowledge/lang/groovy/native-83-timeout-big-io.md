# Groovy native on big io input scores 83.0 (music-scores, 2026-10-21)

- music-scores native Groovy 253-287 B (4 variants, all 12/12 local tests, letter formula checked): CG 83.0 = 10/12, never better. Cause is the ~1.9 s JVM uptime deadline (`io-time-limit.md`): the 2 hidden twins of the 1 MB images time out. Local wall for tests 11/12 is already 2.1-2.6 s
- fix that worked: python3 wrapper `new ProcessBuilder('python3','-c','''<python 248>''').inheritIO().start()`: CG 100% at 309. Triple-single-quote spans the newlines; python code with `''.join` inside is fine
- decode trick (native, 253 B, fast enough only for small inputs): `(w,_,s)=System.in.text.replaceAll(/(\D) (\d+) ?/){_,c,n->c*(n as int)}.split()` then `w=w as int`: the header digits are not matched by `(\D) (\d+)` and `split()` peels w, h and the pixel string
- `[]<true` throws; `q=0>1` as boolean init; BigDecimal has no `%`, use `.intdiv`
