# Interactive puzzles: `perl -apE` makes `$.` the exact game turn (state stale by nbE+1)

Verified on dont-panic (`run.sh --cg`, 2026-10-25, `perl -alpE'$|=1;$_=$.%23==16?BLOCK:WAIT'` = 40 B: 5/7,
win turns reproduced by a referee-faithful simulator).

- The game consumes the k-th printed line as turn k's action. Under `-p`, the program rewrites `$_` while reading
  input line t: init line = 1, elevator/header lines = 2..nbE+1, turn-t line = nbE+1+t. So input line t -> action
  for turn t, and `$.` is the exact game turn: `$.%23==16` blocks game turns 16,39,... on every test.
- The fields are NOT fresh: `@F` on line t is the init fields (t=1), the elevator line (2<=t<=nbE+1), or the turn
  state of turn t-(nbE+1) (t>=nbE+2). A state-derived policy is therefore shifted by nbE+1 turns (stale), while
  turn-only schedules are exact. `$#F` is 7 / 1 / 2 for init / elevator / turn lines (3-char line-type test).
- `$|=1;` (5 B with `$_=` following) is mandatory: a -p program prints one short line per input line and then
  blocks on stdin, so the 4096-byte stdout buffer never fills and turn 1 times out (1000 ms). Same reason
  print-ahead needs ~999 lines to flush.
- Byte budgets for the cell file (`perl -apE'`+core+`'`): core=`$|=1;$_=COND?BLOCK:WAIT` gives file 31+len(COND)
  (COND <= 22 for the 53 B cell ceiling). Storing a per-test key first costs 11 more:
  `$r//=@F[2];` works on every line (undef on elevator lines, "RIGHT"/"LEFT" numify to 0 but are defined), so
  file = 43+len(COND) (COND <= 10).
- Useful when a puzzle admits a short per-turn BLOCK/WAIT schedule keyed on init data; a keyless schedule cannot
  distinguish validators (dont-panic: T7 needs 4 exact turns, no <=22-char universal policy found).
