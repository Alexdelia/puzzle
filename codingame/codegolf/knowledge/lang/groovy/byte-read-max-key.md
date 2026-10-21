# Groovy `max{System.in.read()}` byte argmax

Verified local referee 15/15 (the-descent), CG scored 79.976 twice (4 of 5 validators, cause unknown, probably first-turn JVM timing, see interactive-first-turn.md):

- `for(;;)println((0..15).max{System.in.read()}>>1)` = 48 bytes. Groovy `max{}` with a 1-param closure calls the key once per element, in order (also true for `(0..7).max{s.readLine()}`, 64 bytes). Heights are digits, newline byte 10 < digit bytes, so max index over 16 bytes >>1 is the mountain
- `println (0..15).max{..}>>1` (drop parens) parses as `println(0..15).max..`: invalid
- string literal `'0011223344556677'.max{System.in.read()}` works as a range replacement but is longer (54)
