# Base-B printer on a float quotient

Verified on CG (mayan-calculation perl 170, 2026-10-22).

- `1while s/^/$G[$r%20]/,0|($r/=20);print`: `$r` keeps its fraction, `%` uses only the integer part of a float operand, `0|` stops at quotient < 1; prints one glyph for 0 (do-while shape)
- same length as `$r=$r/20|0`; both are float, wrong above 2^53 (see [integer-division](integer-division.md))
- no `-p`: `s/^/X/` on undef `$_` works, a final `;print` (6) beats `#!perl -p⏎` + `$_="";` (16)
