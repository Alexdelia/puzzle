# Perl: jump to the other of two markers (verified locally + CG, blunder-episode-1 perl 271)

- `$t-=1-pos while/T/g` = sum of the 0-based indexes of all `T` in `$_` (pos is index+1), 19 bytes
- then `$p=$t-$p if/T/` jumps to the other `T`: 33 bytes vs `($p)=grep$_-$p&&$m[$_]=~T,0..@m if/T/` 37
