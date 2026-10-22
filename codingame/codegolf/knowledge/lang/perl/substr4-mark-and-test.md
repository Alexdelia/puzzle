# 4-arg substr: mark a grid cell and test its old char in one expression

`substr$_,31*$F[0]+$F[1],1,R` writes `R` at row/col of a flat `\n`-joined grid (width 30 + newline) and returns the old char, so a test needs no `$&` or capture:

    ($s|=C eq substr$_,31*$F[0]+$F[1],1,R)

replaces `s/(.*\n){$F[0]}.{$F[1]}\K./R/;($s|=$&eq C)`: -5 bytes (-6 inside a wrapper string that doubles `\K` to `\\K`). Contains spaces: not for whitespace-free wrapper cores (scala `Process("perl -anE<core>")`). Offset beyond the string is fatal for 4-arg substr, keep it on fixed-size grids.

Also: inside a `-n` loop, `read ARGV,$_,N` works like `read STDIN,$_,N` (ARGV is the open `-`), -1 byte.

Verified: the-labyrinth perl 210/209, local tests + hidden 16/16, 210 accepted on CG (2026-10-22).
