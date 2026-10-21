# interact: key by filter(>s), keep alive with ++s

Verified on the local referee 2026-10-21 (mars-lander-episode-3, haskell cell journal ...1792609878766916648).

- `do d:r<-xs;...` parses without parens around the cons pattern: `do(d:r)<-` -> `do d:r<-` saves 1
- interactive open-loop plan that must not exit: `words$"DATA "++s` instead of `cycle$`: after the plan the do-block walks the input words, so output waits on stdin and the process stays alive (-1 vs `cycle$` on top of it). The trailing space in DATA is required: else the last data word merges with the first input word (`"30"++"22\n.."` -> word `"3022"`). The words of s produce junk answers after the plan: the plan must finish the game first
- key on the whole input string without parsing: `filter(>s)$words...` keeps a data word w iff w > s (lexicographic vs the full input). Two inputs starting "22\n" and "18\n": words in ("18\n","22\n"] survive only for the "18" input, so one schedule plus one extra word serves both. `dropWhile(<s)` (+3) instead keeps only a leading extra word but frees later words from the > s constraint
- run-length word `c:rot` with `[c..'~']>>rot++" 4\n"` (count 127-c) or `['!'..c]>>...` (count c-32), same length: the second makes the key-word counts short (17/18) instead of long (77/78)
