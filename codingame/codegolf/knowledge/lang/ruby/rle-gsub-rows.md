# RLE decode and flag tricks (ruby, music-scores 191, verified locally)

- `s=gets.gsub(/(\w) (\d+) ?/){$1*$2.to_i}` expands "B 3 W 2" to flat string; second line only
- first line `w=gets.to_i` (parses prefix), height `h=s.size/w` (trailing newline stays in s, floor division absorbs it)
- prev-iteration flag without declaration: global `$q` is nil by default: `$q||y&&push;$q=y`
- `(a..b).find{s[_1*w+x]<s[_1*w+10]}` first row black where staff column white; inclusive `..h*6/7` ok
