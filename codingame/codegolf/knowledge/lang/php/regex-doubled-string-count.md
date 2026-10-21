---
name: regex-doubled-string-count
description: php: count items whose token appears anywhere (before or after) with one preg_match_all on s."\n".s and a lookahead through the blank line
---

`<?=2+preg_match_all('/(\d+) (?=.*\n\n.* \1\n)/s',($s=`cat`)."\n$s");` (64 bytes with raw newlines, CG 100% on dwarfs-standing-on-the-shoulders-of-giants with the `2+L` key).

- joining the input to itself with a blank line (input ends with `\n`) lets a lookahead see every line; matches in the second copy cannot see `\n\n` ahead, so they never count
- `` `cat` `` reads all stdin in php (5 bytes, works on CG) vs `fread(STDIN,99)` (15)
- `<?=` with the expression inlined saves `echo`
- `(\d+) ` needs no `^`: leftmost match starts at the full number, the trailing space blocks mid-number starts
