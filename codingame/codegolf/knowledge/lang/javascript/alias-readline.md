# alias readline

`t=(R=readline)(R(l=R()))` replaces `t=readline(readline(l=readline()))`: callee evaluated before args so `R` is defined for the inner calls. With 4 `readline` uses saves 17 B (ascii-art 124 -> 107, verified local 13/13).
