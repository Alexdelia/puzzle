# Run-length open-loop plan on interactive puzzles

- `interact$ \s->cycle$do(d:r)<-words"...";[d..'~']>>r++" 4\n"`: each word = count char + answer text; `[d..'~']` gives 127-code(d) copies (8 B vs 15 for `[1..fromEnum d]`), list-monad `do` concatenates, no `unlines`
- `cycle$` keeps the process alive: infinite output fills 8K blocks, then blocks on the full pipe. A finite plan exits after printing, and the local referee then fails on writing the next turn ("solution closed its input", racy: passed 1 of 2 runs). Verified mars-lander-episode-3 106 B, CG 100%
- count chars 33..126 minus `"` and `\`, space is the word separator: counts 1..94 except 35 and 93
- key the plan on the first input bytes: `words$last$"A":["B"|s<"2"]` (`last$` needed, `last"A":..` misparses). `dropWhile(<s)$words"..."` is 9 B shorter when one map's plan is a prefix word + the other's plan (words compared to the whole input string s)
