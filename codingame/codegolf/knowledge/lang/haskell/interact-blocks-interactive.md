# interact on interactive puzzles

- `interact` output is flushed at exit only; the process exits without reading the rest of stdin if the output does not depend on it, so printing a whole plan at turn 1 works (CG-verified on vox-codei-episode-1, 238 B)
- forcing the whole input (`length s`, `sum s`) blocks: the game waits for our output. Take only what the first chunk holds
- parse the leading numbers lazily: `map fst$reads=<<words s` skips grid rows (reads fails) and yields `w h R B ...`; bind with a pattern guard `f s|(w:_:r:b:_)<-...=` (shorter than let)
- `words` as table splitter: 1 B separator vs `\n` 2 B; non-ASCII source chars (UTF-8 2 B) accepted by GHC 8.4 in string literals, `fromEnum` gives the code point
