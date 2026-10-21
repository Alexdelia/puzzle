# PHP: one fread + sscanf by reference, references in foreach, expression-only loops

Verified on CG (shadows-of-the-knight-episode-2 php 279, #1, 100%):

- interactive init lines arrive in one chunk: `sscanf(fread($f=STDIN,99),"%d%d%*d%d%d",$W,$H,$x,$y)` reads the whole header (`%*d` skips a field, newlines are whitespace); then `fgets($f)` per later line
- `foreach([[$W,&$x],[$H,&$y]]as[$S,&$p])`: 7.3 reference destructuring, `$p` aliases `$x` then `$y`, so `"$x $y"` prints the live position
- `for(init;cond;body)` with a comma list as body: no braces, `print"..."` (not echo) works inside the list
- assignments inside operands: `($q=$a-$b)<0|$q>9&&$q=...`, `$s=$p+$q-=$q==$p`, `$h=$S+~$l=0` (sets `$l=0` and gives `$S-1`)
- `$p+($p=$q)` reads the new `$p` twice (gives `2q`): save the old value first
- SAME/WARMER/COLDER first-char test without `[0]`: full line string compare `$r>T` (bare `T`) splits W from C/S, `$r[5]` is falsy only for `SAME\n`
- bool `^`: `a>b^c>d` is `!=` of two comparisons, 3 bytes shorter than `(a>b)==c>d`
