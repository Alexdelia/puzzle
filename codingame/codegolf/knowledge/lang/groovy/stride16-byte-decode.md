# Stride-16 byte decode of hardcoded cells

Verified 2026-10-21 vox-codei-episode-1 (CG 100%, 222 -> 205 B).

- cell code = y*16+x as one byte in a slashy string: `for(c in /DATA/.bytes)println c>1?c%16+' '+(c>>4):'WAIT'` (byte 1 = WAIT, cells (0,0) and (1,0) unusable). Replaces `c%12+' '+(c.intdiv(12)-2)` with offset 24: -10 B. Rows y<=7 only (byte <= 127; 2-byte UTF-8 breaks `.bytes`)
- NUL byte in a slashy string is a groovyc parse error; byte 1 and `$` followed by non-identifier char (`$\x01`, `$4`) are fine
- packing: CSP over (buffer size S, modulus m, string length L) with all min covers x permutations per hidden test, forward checking per test; only 3 chars after the plan must be safe (game ends at the win), safe = WAIT or a bomb on a `.` cell. 52 chars total cost (S + m digits + L) vs 58 with fixed plan order
