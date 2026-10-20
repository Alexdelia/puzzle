# Argmax of N one-char lines with coreutils

Verified on the-descent (CG): `sed 8q|nl -v0 -w1|rev|sort|tail -c2` prints the 0-based index of the max line (ties: highest index) followed by a newline.

- `sed 8q` beats `head -8` by 1 and stops after one turn on an interactive pipe
- `nl -v0 -w1` gives `i<TAB>h`, `rev` makes it `h<TAB>i`, so a plain `sort` orders by height then index and `tail -c2` is `i` plus newline, no `cut`
- `split -8 --filter=...` is lazy: the chunk closes only when the next line arrives, times out on interactive input
- `grep -nm8 .` replaces `sed 8q|nl -v0 -w1` in 6 fewer bytes but numbers from 1: fixing that costs more
