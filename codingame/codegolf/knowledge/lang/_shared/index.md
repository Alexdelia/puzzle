# Shared: CG facts valid in every language

- [length](length.md): length = UTF-8 bytes, trailing newlines free, unicode packing useless, verified
- [output-compare](output-compare.md): exact line match, trailing spaces and CR fail, missing final newline and ONE trailing empty line free, two or more trailing empty lines FAIL (local referee-core enforces it too), verified
- [io-time-limit](io-time-limit.md): io puzzle run limit about 6000 ms per test in Python, startup included; per language (Groovy ~1.9 s), verified
- [memory-limit](memory-limit.md): about 768 MiB peak RSS per test, over it = fail (Python killed silently, Haskell shown as timeout), local referee does not check it, verified
- [crash-exit](crash-exit.md): io puzzles ignore exit status, crash after output passes, verified
- [best-kept](best-kept.md): CG keeps best submission per language, longer submit harmless but unconfirmable, verified
- [system-wrapper](system-wrapper.md): `os.system("bash -c ...")` runs bash on CG, interactive io works; cell length = wrapper + shortest core, verified on Python; TRAP: spawned `perl` locally = host 5.42, CG = 5.34 (variable-lookbehind bug), test with `PATH=$CG_PERL/bin:$PATH`
- [hidden-validators](hidden-validators.md): hardcode/exploits allowed (user rule), `submit.sh --probe`, best-kept makes each probe a 1-bit oracle when shorter than our row, local validators synthetic; first submit in a no-row language reads passed/total via the partial row score (or sum of weights: scores can be weighted)
