# n newline-ended lines with print(*[s]*n,sep="\n")

`print(*[s]*(c//5-8),sep="\n")` prints n copies of `s`, one per line: n-1 separators plus the
final newline. Consecutive `print` calls then flow without a blank line between them, which the
string form `print((s+"\n")*(c//5-8),end="")` also does but costs 2 B more (verified
blunder-episode-1 python 213 B CG 100%, 2026-10-25).

Trap: n=0 prints one empty line (`print(*[],sep=...)` still emits the final newline), unlike the
string-multiply form which prints nothing. Safe only when 0 never occurs at runtime (an unused
lookup slot, a hash residue no graded input hits).

Single-use stdin: inline the read, `sum(open(0,"rb").read())`, instead of `s=open(0,"rb").read()`
plus `sum(s)`; the assignment line plus the one use cost 4 B more (same submit, verified).
