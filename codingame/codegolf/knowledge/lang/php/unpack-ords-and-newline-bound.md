# php: ord loop and free newline

Verified 13/13 local, CG ascii-art 135 (2026-10-21).
- `foreach(unpack('C*',$t)as$c)` gives ints, saves `ord()` per use over `str_split`
- `~-x` = `x-1`, `~-min(..)*$l` shorter than `$l*min(..)-$l`
- keep trailing "\n" of T in the loop: row lines end with "\n" so `substr($r,27*$l,$l)` is "\n"; bound `min(y?:27,27+($c<32))` maps it there, no chop and no `print"\n"`
- `[$l,,$t]=$f=file('php://stdin');for(;$r=$f[++$i+2];)`: rows without array_slice; `/dev/stdin` fails local sandbox
