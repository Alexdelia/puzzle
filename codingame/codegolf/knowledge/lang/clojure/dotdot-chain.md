# Clojure: `..` for method chains

- `(..(ProcessBuilder.[...])inheritIO start)` is 1 B shorter than `(.start(.inheritIO(ProcessBuilder.[...])))`: `..` threads the object through bare method names, no `.` prefix per call, one paren pair for the chain. Verified local 15/15 and CG 100 % (super-computer 115 rank 1, 2026-10-21)
- general: k zero-arg methods cost sum(len)+3k nested, sum(len)+k+3 with `..` (separator after `)` of the target is free): saves 2k-3, so k>=2
- no spaces needed: `ProcessBuilder.["sh""-c""..."]` (vector right after the ctor symbol, adjacent strings)
- `sh -c` core works like the perl wrapper (`perl-wrapper.md`): `["sh""-c""<shell core>"]`, core may use `'` but no `"`
