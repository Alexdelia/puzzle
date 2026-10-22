# Shared suffix of alternative words goes into the format

When a printed word is one of several barewords with a common suffix (`LEFT`/`RIGHT` end in `T`), put the suffix once in the `printf` format and print the stems: `printf"%d %d %sT\n",$x,$y,$l?LEF:RIGH` instead of `"%d %d %s\n",...,$l?LEFT:RIGHT`. Saves (number of words - 1) x suffix length - suffix length... net -1 for LEFT/RIGHT (two words, suffix 1). Stems stay valid barewords (no `use strict`).

Verified: the-fall-episode-3 perl 570 -> 569, CG accepted 2026-10-22.
Same idea in any language with a format string or a concatenation (`+"T"`), when the words are already literal alternatives.
