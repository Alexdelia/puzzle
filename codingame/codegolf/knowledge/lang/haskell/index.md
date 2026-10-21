# Haskell

- [env](env.md): GHC 8.4.3 exact, base 4.11, CPP ok, getLine fine without final newline, verified
- [regression-golf](regression-golf.md): Double length via sum(1<$x), helper g for slope, !!min 7
- [group-unwords-runs](group-unwords-runs.md): Data.List group + unwords for run-length output, bit extraction without imports, lexing traps, verified
- [char-index-clamp](char-index-clamp.md): glyph index 0..26 from fromEnum with mod/max/min, no import
- [visited-list-state](visited-list-state.md): visited-position list instead of grid mutation, last(x:[y|c]) as if, lexing traps
- [thor-counter-interact](thor-counter-interact.md): lazy interact + list-comprehension optional letters, 113 on power-of-thor
- [section-shares-median](section-shares-median.md): (e-) shares e at -O0, doubled-list median, transpose column split
- [music-scores-columns](music-scores-columns.md): g n chunker reused for rows and RLE pairs, Bool pixels, transpose columns, zip3 with shifted self (337 CG 100%)
- [system-wrapper](system-wrapper.md): import System.Process;main=system"..." 35 bytes overhead, super-computer 77
- [heap-morse-codes](heap-morse-codes.md): lazy heap-order Morse code list, the-resistance 347 (slow on big validator)
- [intset-flood](intset-flood.md): IntSet persistent flood fill, fmap not map, 242 on surface
- [perl-wrapper-hardcoded](perl-wrapper-hardcoded.md): system wrapper around perl core, string escapes (\\K, \"), the-labyrinth 268
