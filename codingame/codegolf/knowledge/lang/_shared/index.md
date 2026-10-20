# Shared: CG facts valid in every language

- [length](length.md): length = UTF-8 bytes, trailing newlines free, unicode packing useless, verified
- [output-compare](output-compare.md): exact line match, trailing spaces and CR fail, missing final newline and ONE trailing empty line free, two or more trailing empty lines FAIL (local referee-core still lenient), verified
- [io-time-limit](io-time-limit.md): io puzzle run limit about 6000 ms per test, startup included, verified
- [crash-exit](crash-exit.md): io puzzles ignore exit status, crash after output passes, verified
- [best-kept](best-kept.md): CG keeps best submission per language, longer submit harmless but unconfirmable, verified
- [system-wrapper](system-wrapper.md): `os.system("bash -c ...")` runs bash on CG, interactive io works; cell length = wrapper + shortest core, verified on Python
