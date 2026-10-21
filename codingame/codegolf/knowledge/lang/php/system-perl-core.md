# system() + perl core (dont-panic, php 100)

`<?system('perl -anE\'<core>\'');` verified on CG (don't-panic 100%, 100 bytes, was 704 hardcode). Single-quoted PHP string: `\'` costs 2 bytes in total, a double-quoted string would need `\$` for every perl `$`. No closing `?>`/newline needed; short tag works. `system` flushes per call and the child inherits stdin/stdout, interactive turns work.

Core 80 bytes: `%e=(@F[0,1,3,4],%e);say+($F[1]-$e{$F[0]})*(/R/-.5)>0?BLOCK:WAIT if$|=@F==3`. `/R/-.5` (sign of direction, matches RIGHT only; NONE line has no R) is 4 bytes under `($F[2]cmp M)`.
