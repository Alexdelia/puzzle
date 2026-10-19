# Shared: CG facts valid in every language

- [length](length.md): length = UTF-8 bytes, trailing newlines free, unicode packing useless, verified
- [output-compare](output-compare.md): exact line match, trailing spaces and CR fail, trailing empty lines and missing final newline free, verified
- [io-time-limit](io-time-limit.md): io puzzle run limit about 6000 ms per test, startup included, verified
- [crash-exit](crash-exit.md): io puzzles ignore exit status, crash after output passes, verified
