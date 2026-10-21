# php: bit-string via fgetc loop, one-pass run regex

- `for(;10<$c=ord(fgetc(STDIN));)$s.=sprintf("%07b",$c);` stops at newline/EOF; `10<$c=...` needs no parens in php (verified php 7.3.9)
- `preg_replace("/(0)+|1+/"," 0$1 $0",$s)`: unmatched `$1` is empty, `$1` and `$0` stay literal in double quotes (verified)
- `ltrim(x)` drops a leading space 3 bytes shorter than `substr(x,1)`
- php `unpack` has no `B` format (verified 7.3.9)
- chuck-norris php 117 with these
