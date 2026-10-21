# PHP

- [env](env.md): PHP 7.3.9 CLI, no ini, short_open_tag on, notices to stderr, no zlib/gmp, verified
- [flat-string-walk](flat-string-walk.md): grid walk as mutable string, bare constants, unset vars as 0, xor priority loop
- [chuck-norris-loop-regex](chuck-norris-loop-regex.md): fgetc/ord loop with `10<$c=`, one-pass run regex, ltrim, no unpack B
- [regex-doubled-string-count](regex-doubled-string-count.md): preg_match_all on s."\n".s with lookahead past the blank line, `cat` backtick for stdin
- [bare-constants-strings](bare-constants-strings.md): undefined constants as strings, for(;$x++<$w;), flat column walk, trim output
- [null-counter-compare](null-counter-compare.md): uninitialised `$i` compares as bool; use `++$i<=` in first test, no init
- [unpack-ords-and-newline-bound](unpack-ords-and-newline-bound.md): unpack C* loop, ~-x, newline of T kept via bound 27+($c<32), file() destructure
- [quiet-notices](quiet-notices.md): hot-loop notices made a 100%-local port score 84% on CG; error_reporting(0); const arrays avoid global
- [const-define-globals](const-define-globals.md): const/define replace `global`, printf with bare constants, +$x on exploded lines
- [recursive-flood](recursive-flood.md): userland recursion unbounded in 7.3, flat string flood fill, header explode, query regex
- [temperature](../../puzzle/temperature.md): php 91 = `9e3` sentinel + explode(" ",explode("\n",`cat`)[1]) loop; N=0 token "" gives 0 for free
- [system-perl-wrapper](system-perl-wrapper.md): <?system('perl -e..\'..\'') = core+15 bytes, beats backticks/nowdoc
- [sscanf-fread-by-ref](sscanf-fread-by-ref.md): one fread header + sscanf by ref, `[$S,&$p]` foreach refs, comma-list for body, inline assignments
- [system-perl-core](system-perl-core.md): <?system('perl -anE\'...\''); wrapper for interactive puzzles, 704 -> 100 on dont-panic
- [print-all-answers-upfront](print-all-answers-upfront.md): fixed answer sequence printed at turn 0 in one loop, exit without reading further turns (CG verified)
