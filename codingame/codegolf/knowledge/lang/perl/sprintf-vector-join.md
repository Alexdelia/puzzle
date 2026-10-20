# sprintf `%*v07b` with empty join

`$_=sprintf"%*v07b","",$_` = 7-bit binary of every char of `$_` concatenated, no per-char `s///ge`. 1 byte under `s/./sprintf"%v07b",$&/ge`. Needs `-l` (else the newline adds `0001010`). Verified chuck-norris 14/14.

Trailing separator trick: header `0$1 ` + body + ` ` per run then `chop` (4) beats leading space + `s/.//` (5).
