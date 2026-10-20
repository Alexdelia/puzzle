# perl one-liner tricks from music-scores (bash 325B)

All verified in `perl -0ne'...'` under the CG bash toolchain (perl 5.34):

- `-0ne` slurps stdin into `$_` (no 777 needed without NUL bytes); `$w=$_+0` reads the first number of the first line
- `s/(\D) (\d+)/CODE/ge` with `\D` skips a leading `W H` header line (digit before the space), CODE may be a statement with a `for` modifier: `$c[$i++%$w].=$1for 1..$2` transposes an RLE image into column strings in one pass
- `y/BW/QH/` on the whole input before decoding makes output letters come straight from pixels
- string xor `($a^$b)=~/\31(.*\31)?/`: `\31` is octal in a regex when fewer than 31 groups exist (saves `\x19`), `^` of two equal-length strings gives NUL where equal; the match on an rvalue in parentheses works for a single `m//`, not for `m//g` loops
- index from a boolean: `$t[-($x>$y)]` = last element if true else first
- float `%`: `(x)%7` truncates to int, so `chr(71-(x)%7)` replaces `substr("GFEDCBAGFEDC",x,1)`
- `$a`/`$b` may be read inside a `sort{}` block even when the script assigns `$a`; inside `-0ne'...'` avoid `'` (use `join$,,` for `join''`)
- `for(@c,$S){...}` sentinel: append a never-flagged element to flush a trailing group (not needed if input ends with margin)
