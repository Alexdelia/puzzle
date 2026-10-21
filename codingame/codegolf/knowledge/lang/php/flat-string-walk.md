# PHP flat-string grid walk (blunder-episode-1, 367 bytes, verified on CG)

- `stream_get_contents(STDIN)` needed: `fread(STDIN,n)` on a pipe returns <= 8192 bytes per call
- strings are mutable by offset (`$s[$p]=' '`), grid = whole stdin with header kept
- row width without parsing: `1+strstr($s,' ')` ("15\n..." numeric prefix, notice only)
- bare constants as strings (`SOUTH`, `T`, `LOOP`, `X`) cost only stderr warnings on CG
- unset vars as 0 (`$x^=3`, `substr('X#',$b)`, `$n++<999`) work
- first-free direction pick: `for($j=0;blocked($k);)$k=$j++^$x;` with dirs S,E,N,W and `$x` 0 or 3 (inverted)
- modifier set: `($m=strpos("SENW$c",$c))>3||$k=$m`
