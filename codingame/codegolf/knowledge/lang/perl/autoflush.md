# autoflush in interactive puzzles

Perl stdout to CG is block-buffered: `print"0 4\n"while<>` times out on turn 1 (Mars Lander 3, CG test game 910370073). `$|=1;` in front fixes it (test game 910370099 ran 107 turns). Verified 2026-10-20. Costs 5 bytes (`$|=1;` or `$|++;`) in every interactive Perl cell.
