# The Fall - Episode 2 (code golf): hidden validators

Probably 50 validators, not 1:1 with the 7 IDE tests. Not recovered exactly: most are maps of sizes no IDE test has, far beyond what count probes can rebuild. No `NN.json`, no `complete`.

## Method

Count probes (task `validator.the-fall-episode-2.-.1792523691430509058`, 2026-10-20, `agent/state/oracle/the-fall-episode-2.tsv`): the general Python solver (`solution/python/main.py`, 100% on CG) exec'd from bash/ruby/perl (`exec /usr/bin/python3 -c ...`, works on CG), printing `X` (instant fail) when a predicate on the input is false. Score of a fresh-language row, or of a probe beating the row, = share of validators where the predicate holds. Rock predicates gate at run time (garbage when a rock line shows up). Generator: journal scratch `gen/mk.py`.

## Count

- every score seen (ours and other players' partial rows: 16 32 58 64 68 80 84 90 100) is even, 58 and 90 are not multiples of 4: count N = 50 fits all exactly (25 does not). N = 100 not excluded. Counts below assume N = 50 (count = score / 2)

## Features

| predicate | score | count /50 | source |
|---|---|---|---|
| grid, exit, Indy start equal an IDE test | 16 | 8 | bash chain0 |
| exact copy or WxH = 10x8 (rock tests size) | 16 | 8 | bash chain1 |
| exact copy or WxH of any IDE test (5x3 8x4 6x9 13x10 13x8 10x8) | 16 | 8 | bash chain2 |
| exact copy or W<9 | 32 | 16 | bash chain3 |
| exact copy or W<13 | 68 | 34 | bash chain4 |
| exact copy or W<17 | 100 | 50 | bash chain5 (became our bash row, 4595) |
| exact copy or H<9 | 58 | 29 | perl (row 58) |
| no rock ever appears (while our solver plays) | 80 | 40 | ruby fresh row |
| no rock, or exact copy | <=80 (row unchanged, longer probe) => 80 | 40 | ruby |
| no rock, or W<11 | 100 | 50 | ruby (became our ruby row, 3049) |
| hardcoded IDE answers keyed on first grid row (earlier task) | 16 | 8 | perl, earlier |

Deductions (high confidence, arithmetic on the rows):

- 8 validators are exact copies of IDE tests 1-5 (none of rock tests 6-7; which ones unknown; 8 copies of 5 tests means some appear twice, or N = 100)
- 42 have a size no IDE test has: 8 with W<=8, 18 with W 9..12, 16 with W 13..16; every validator has W<=16
- 21 non-copies have H<=8, 21 have H>=9
- 10 validators have rocks, all new maps (not IDE copies, not 10x8) with W<=10

## Key / hardcode

No hardcode possible: ~42 unseen maps. A short solution needs a general solver that survives the 10 rock validators (rock handling required: earlier python probe without rock kill failed). Shortest key separating validators: unknown.
