# Music Scores

pretty_id `music-scores-codesize`, leaderboard `music-scores-codesize`, io.

## Rules (as referee)

- line 1 `W H`, line 2 RLE `B n W n ...` over the flat image, row-major; both lines end with newline in all 12 tests
- output: notes left to right, space separated, `<letter><H|Q>`, no distinction between low and high C..G
- notes possible: low C (ledger line below line 5) up to G above line 1, 12 positions half a spacing apart
- io limit: referee uses core default 6000 ms (not probed); plain Python reference takes 160 ms max

## Input facts (verified on the 12 tests)

- sizes: 120x176 (tests 1-4), 390-520 x 176 (5-9), 420x78 (10), 4432x241 (11), 4020x241 (12)
- test 11 is 69871 chars, test 12 81022: `capture-io.sh` truncates them (CG stderr cut near 4000 chars); `referee/capture/capture.sh` captures zlib+base85 in 3800-char chunks
- staff lines span from a left margin; first black column of the image holds only staff line pixels, so rows black there = the 5 lines
- line thickness 1 (test 10) to 4; spacing 24 in tests 1-9
- the ledger line of a C is wider than the head and in test 11 touches neighbour notes: splitting notes by blank columns works only after masking the ledger rows (line 5 rows shifted by one spacing)
- test 9 "Very close": tail up only ~1.7 spacings long, so a column-extent < 2 spacings test misclassifies the tail as head; tail columns = columns whose non-staff black count is >= 0.75 of the max in the note
- half note heads are hollow; on a line the staff line crosses the hollow, on low C the ledger crosses it: test fill on the first non-ruled row next to the head center

## Traps

- statement example matches test 1 layout (120x176, line thickness 4)
- no extra symbols (clef, bars) in any test image

## Image facts for golf (verified on 12 tests + 10 validators)

- hollow (half) head ERASES staff and ledger lines inside it: the head center pixel is white for H, black for Q, even for heads on a line
- a hollow head border can coincide with a staff line: a column through the head may show only one border (test 10), so top/bottom of the mid column is not the center
- the head edge column opposite the tail (leftmost column of a tail-up note, rightmost of a tail-down note) holds a short run centred on the head center; the tail end column holds a long run
- low C ledger is wider than the head: it touches (test 11) or overlaps (test 12, validators 9, 10) the columns of neighbour notes
- tails: up on the right side, down on the left side

## Shortest known (method, not yet golfed per language)

Verified 22/22 on local referee, Python prototype 502 bytes (far from top python 206: leaders use something cheaper, not found).

1. decode RLE to flat string, columns `s[x::w]`; staff column `S` = first column holding a black pixel
2. `a`,`r` = first/last black row of `S`, `k` = black count / 5 (line thickness), spacing `q=(r-a+1-k)//4`; truncate every column (and `S`) at the ledger top `l=r+q-k+1`. Cut must be exact: `l=r+(r-a)//6..8` fails (16/22)
3. notes = maximal runs of columns with `'B' in c and c != S` (blank margins excluded by the `'B'` test)
4. center row `y`: of the two end columns of the run, take the one with fewer rows differing from `S`; `y` = mid of its first and last differing row
5. letter `"FEDCBAG"[round(8*(y-a)/(r-a))%7]`; type `H` iff the run's middle column is white at row `y`

Needs: string slicing with step (or 2D index), no regex, no big ints.

Rejected:
- mid column top/bottom diff as center: hollow border merged into a line (test 10)
- group by black count > staff count, or by diff count > line thickness: hollow heads split into several groups
- no ledger handling: notes merge in tests 11, 12, validators 9, 10
