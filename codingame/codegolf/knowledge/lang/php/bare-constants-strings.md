# Bare constants as strings (verified on CG, music-scores php 256, 100%)

- undefined constants (`B`, `AGFEDCB[...]`, `QH[...]`) act as strings in PHP 7.3 with a warning on stderr: CG accepts, saves the quotes
- `if(!$p&$p=$k)` parses as `!$p&($p=$k)`: reads old `$p`, then assigns
- `for(;$x++<$w;)` runs x = 1..w, saves 3 B vs `for(;$x<$w;$x++)`
- flat-string column walk: `$i=$x;...$i+=$w`, `$i-$x` = y*w, so divide by strlen instead of h (`15*($i-$x-3*$w)/$h%7`, float `%` truncates)
- trailing space in the output fails the referee: join via `$o.=" ".X` and `echo trim($o)`
- `fgets(STDIN)+0` reads the first int of "W H" (notice only)
