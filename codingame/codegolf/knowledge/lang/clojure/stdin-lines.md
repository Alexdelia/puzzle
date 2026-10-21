# Clojure: read all input lines

- `(line-seq *in*)` throws ClassCastException (`*in*` is a LineNumberingPushbackReader, not a BufferedReader), verified locally 2026-10-21
- `(.split(slurp *in*)"\n")` gives a String[]: destructuring `[w _ t & R]` works on it (24 bytes, shortest found)
- `(Long. w)` parses a line (9 bytes) vs `(read-string w)` (13)
- `(nth(partition w row)i)` + `mapcat` over `T` chars + `(apply str ...)` beats `subs` with computed offsets (189 vs 198); `mapcat` over the string with `(int %)` beats `(map int t)` by 4
- glyph index `(min 26(bit-and 223(- (int %)65)))`: letters of both cases, everything else 26
