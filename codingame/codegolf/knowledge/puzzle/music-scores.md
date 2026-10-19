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
