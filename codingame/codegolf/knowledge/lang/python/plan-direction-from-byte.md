# Plan hardcodes: direction from the data byte, not from enumerate

For an interactive plan-hardcode (one rotation per line), `for i,c in enumerate(D): ... i&2and'LEFT'or'RIGHT'`
costs 12 B more than `for c in D: ... c&32and'LEFT'or'RIGHT'`: the predicate on the byte is 1-4 B longer than
`i&2`, the `enumerate(` machinery is 12 B. The catch is the plan bytes must be chosen so the predicate yields
the direction each rotation needs.

- `and'LEFT'or'RIGHT'` (21 B) / `and'RIGHT'or'LEFT'` (21 B): pick the one whose truthy branch is shorter if the
  predicate values allow.
- pick the coordinate base/orientation so the 4-cycle rotations hit the predicate correctly and no byte needs
  escaping (`'`/`"`/`\`/LF/CR/>=128 raw bytes are invalid or costly). Base b swapped means `c=b*y+x`, printed
  `c%b,c//b`.
- 2-cycle rooms (2<->3, 4<->5) are direction-free; for a repeated rotation of one 4-cycle cell the net 2 steps
  is the same for RR and LL (180 deg), so pairs are free too.
- a single 4-cycle rotation that needs the opposite net can be done as 3 rotations in the predicate direction
  (+2 data bytes) if the intermediate types are harmless (simulate).
- known-good example (the-fall-episode-3 python 119): base 13 swapped, P=`c&32` (22 B with the words), 0
  extra rotations, every byte <128 and free of quote/space; a 5-chunk tilde layout was 2 B worse than 4
  chunks + `.split()` + `%5&3`.
- metric reminder: source bytes, not chars; raw bytes <128 are 1 B each, only NUL is stripped by CG.
