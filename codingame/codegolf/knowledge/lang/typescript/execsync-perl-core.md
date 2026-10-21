# execSync + perl flood core (the-labyrinth)

Verified 2026-10-21 (CG 100%, 263 B): `require("child_process").execSync(\`perl -anE'...'\`,{stdio:[0,1]})` with the hardcoded 15x30 perl core of the lua cell. In a template literal perl `\K` must be written `\\K` (+1 B per use); a lone `\K` becomes `K`.

- CG perl rejects a merged lookbehind `(?<=X.|X.{30}.)` (different lengths in one lookbehind): local perl 5.42 accepts it (258 B), CG did not confirm 258 nor 262 (row stayed 430). Keep separate `(?<=X.)|(?<=X.{30}.)`
- `(.*\n)` capture instead of `(?:.*\n)` is free: `$#-` is read from the later until-regex
- `XR|R(X)|(X).{30}R|R.{30}(X)`: leaving the first alternative uncaptured makes `$#-` 0-based, so the move list needs no filler: `say+(LEFT,RIGHT,UP,DOWN)[$#-]`
- `($s|=$&eq C)?y/T/X/:y/?C/X/` saves 1 B over a separate statement
