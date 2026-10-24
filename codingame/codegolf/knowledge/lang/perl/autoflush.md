# autoflush in interactive puzzles

Perl stdout to CG is block-buffered: `print"0 4\n"while<>` times out on turn 1 (Mars Lander 3, CG test game 910370073). `$|=1;` in front fixes it (test game 910370099 ran 107 turns). Verified 2026-10-20. Costs 5 bytes (`$|=1;` or `$|++;`) in every interactive Perl cell.

- `$|` is a boolean on read: `$|=-12`, `$|=15`, `$|=18` all read back `1` (perl 5.34 and 5.42, measured 2026-10-24). It cannot carry a counter, threshold or letter across turns, only enable autoflush; keep the value in a normal global (`$a`, `$;`) and use `$|||=...` only for the run-once guard + flush.
