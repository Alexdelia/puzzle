# Chuck Norris (Unary)

pretty_id `chuck-norris-codesize`, leaderboard `chuck-norris-codesize`, io.

## Rules

- 1 line, `0 < N < 100` ASCII chars, no CR; some tests have no final newline
- each char as 7 bits, runs of same bit: `0 <0*len>` for 1s, `00 <0*len>` for 0s, blocks separated by single spaces
- io limit 6000 ms per test

## Shortest known

- Python 119: `re.findall('0+|1+', ''.join(f'{ord(c):07b}' for c in input()))`, then `'0'*(g<'1')+'0 '+'0'*len(g)` per run
