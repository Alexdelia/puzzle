# Flat-index column scan, music-scores (verified, javascript 227 B, CG 100%)

- column scan on flat string `s` without `h` or `y`: `for(i=x++;i<m&&s[i]>=s[i-x+11];i+=w)` (m = 7/8 of length, x++ makes staff column 10 read as `i-x+11`); letter idx `(13*i-42*w)/m%7|0` (42 or 43 pass 22 tests+validators like 41..43 range; 40, 44 fail), no `x` term needed
- `(R=readline)()` first call then `R()` saves 3 B over two `readline()`
- `print(...o)` joins by space on CG and local (saves `.join` ` ``)
- `x=q=o=[]` initialises counter (`[]<w` true, `x++` numeric), flag (`true>[]` true) and list at once; body-less outer `for` whose update clause does the work saves braces
- chain `i<m>q` = `(i<m)>q`: true only for flagged-now and unflagged-before
- string output with trailing space fails the local referee (and likely CG)
