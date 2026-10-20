# `-nl` first-line ternary + `}{`

`#!perl -nl`, body `$.<2?$s=y/.-/01/r:$h{s/./.../ger}++}{...print` : header line handled by `$.<2?` (assignment in the true branch parses fine), later lines counted, `}{` closes the `-n` loop so the final code runs once without `END{}`. `s///ger` inside a hash key replaces `join"",map{}/./g`. Verified the-resistance 195 vs 219 for `-0a`.
