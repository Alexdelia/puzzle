# Clojure

- [env](env.md): 1.11.1 AOT compiled on Java 8, run on Java 21; `(ns Solution(:gen-class))` required, top-level runs at compile and run, no stdin read at top level without guard, verified
- [read-all-numbers](read-all-numbers.md): slurp+read-string vector parse; println not printf; 0.9 not .9
- [stdin-lines](stdin-lines.md): line-seq on *in* fails; .split of slurp, Long. parse, partition+mapcat glyph slicing, verified
- [golf-tricks](golf-tricks.md): read twice, map-literal switch, remove, indexOf on vec, blank-copy for pair partner, string accumulator (blunder-episode-1 409)
- [regex-replaceall](regex-replaceall.md): .replaceAll with $0, .trim, 7-bit string via toString(+128) (chuck-norris 177)
- [perl-wrapper](perl-wrapper.md): ProcessBuilder inheritIO start at top level runs a perl core, 87 B overhead (mayan-calculation 262)
- [read-pairs-sorted-diff](read-pairs-sorted-diff.md): (read) pairs + apply map list, F=#(map -(sort > %)(sort %)) gives range and deviation sum, prn (network-cabling 167)
