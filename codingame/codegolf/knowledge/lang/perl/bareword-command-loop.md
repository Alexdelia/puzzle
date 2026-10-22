# Loop over bareword command names, deltas by regex

Verified on CG 2026-10-22 (the-bridge-episode-2 perl 277 -> 273, 100%).

When a search tries a fixed set of output commands, loop over the names themselves and derive each numeric effect with letter tests on `$_`, instead of an index range plus a name table:

```perl
map{$v=$s+/E/-/L/;$d=/N/-/UP/;$j=/J/;...push@q,$_.$/...}SPEED,UP,JUMP,DOWN,SLOW
```

replaces

```perl
map{$v=$s-int$_/2;$d=$_%2*$_;$j=!$_;...(JUMP,DOWN,SLOW,SPEED,UP)[$_]...}-2..2
```

- barewords in the `map` list need no quotes (no `strict`); the 30 B name table disappears, each delta costs a few bytes more: net -4 here
- pick a letter (or pair) unique to the command: `E` only in SPEED, `L` only in SLOW, `N` only in DOWN, `UP` (not `U`: JUMP has U), `J` only in JUMP
- a match is 1 or "" so `/A/-/B/` is a signed delta, `!/J/&&X` is "" (0) for JUMP
- TRAP when converting: `(LIST)[$_]` over `-2..2` uses negative indexes, so the old order was SPEED,UP,JUMP,DOWN,SLOW, not the table's written order
