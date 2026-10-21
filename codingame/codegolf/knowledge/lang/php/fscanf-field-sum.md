# fscanf field sum / pick in one expression

Verified on local PHP 7.3.9 (2026-10-21): `fscanf(STDIN,"%d%*d%d")` returns the non-suppressed fields of one line as an array (`%*d` skips a field); a line with fewer fields gives NULL for the missing ones (`"22"` -> `[22,NULL]`). So `array_sum(fscanf(STDIN,"%d%d"))` = X+Y of a line in one expression, `max(...)`/`min(...)` likewise, no destructuring or loop variable.

`<?for(;;)echo array_sum(fscanf(STDIN,"%d%d"))/99-40|0," 4\n";` is 61 bytes; `[$x,$y]=fscanf(...)` destructuring costs 4 more. Plain `fgets(STDIN)/D` reads only the first field (leading numeric). Not yet a CG-scored solution anywhere.
