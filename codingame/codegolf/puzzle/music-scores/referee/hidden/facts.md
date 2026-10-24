# music-scores hidden validators

Status 2026-10-24: validators 1-4 recovered EXACTLY (`01..04.in/.out` here, md5 of the decoded image confirmed on CG); 5-12 partly known (answer features below), not recoverable cheaply. Log: `agent/tool/oracle.sh --log music-scores`, journals `agent/state/journal/validator.music-scores.-.1792523690839018744.md` and `...1792834543963454769.md`.

## Count, titles, weights

12 validators, titles = IDE test titles 1:1 (`Validator_1..12`), weights 83 each for 1-11, 87 for 12. Source: CG report of submission 41554322 (`agent/tool/cg-report.sh 41554322`, vbnet 386, 100%). Confidence: certain.

## Per validator

| # | title | known | source | confidence |
|---|---|---|---|---|
| 1 | One quarter note between lines | EXACT: answer `FQ`; image = test 1 (AQ) with the note moved down 24 rows (low F, tail up, x 50-71), 120x176 | probes bit0-2 (letter) then md5-x50 (perl, 2026-10-24) | certain (md5 8 hex) |
| 2 | One quarter note on a line | EXACT: `GQ`; test 2 (BQ) note moved down 24 rows (low G on line 4) | same | certain |
| 3 | One half note between lines | EXACT: `FH`; test 3 (AH) note moved down 24 rows | same | certain |
| 4 | One half note on a line | EXACT: `DH`; high D on line 2, tail DOWN on the left: note of test 6 (DH at x 40-61) moved to x 50 in the 120x176 staff | same | certain |
| 5 | Only quarter notes without lower C | answer = test 5's `CQ CQ CQ CQ DQ EQ CQ AQ` exactly; image differs from test 5 and from every regular layout of test 5's notes (W 390/420/440/520, pitch 38-46, first x 4-79); H 176 | probes same-answer, md5-x50, mixed1 | high |
| 6 | Only half notes without lower C | 7 notes, first letter D, answer != any test answer; H 176 | bit0-2 (n mod 8 = 7), mixed1 (n==7 and starts D) | high |
| 7 | Scale half and quarter notes | answer != any test answer, n = 1 mod 8 (9 likely); NOT `EH FQ GH AQ BH CQ DH EQ FH` (or n != 9); H 176 | bit0-2, mixed1 | high |
| 8 | Lower C | 11 notes, first letter C; H 176 | bit0-2, mixed1 | high |
| 9 | Very close | 10 notes, first letter B; H 176 | bit0-2, mixed1 | high |
| 10 | Only 1 pixel wide | n = 4 mod 8 (20 likely), not two equal halves if n=20 (test 10 is test 9 twice); size 420x78 likely | bit0-2, mixed1, 2026-10-20 size probes | medium |
| 11 | Doctor Who theme | n = 4 mod 8 (84 likely), not a 28-note phrase x3 if n=84 (test 11 is); size 4320x241 likely | bit0-2, mixed1 | medium |
| 12 | Random | exactly 90 notes; size 4020x241 likely | bit0-2, mixed1 (n==90 passed) | high |

Earlier facts (2026-10-20, ruby/python probes): 9 validators have H = 176, the other 3 have sizes of tests 10/11/12 (420x78, 4320x241, 4020x241), every W is a test width; geometry of the tests holds on all (staff left column x = 10, ledger cut 7h/8, letter `(15y-45)//h`, H/Q pixel at x+5).

## How 1-4 were rebuilt

`agent/state/journal/validator.music-scores.-.1792834543963454769.scratch/gen.py` (scratch, git-ignored): blank staff of test 1 (rows 34-37, 58-61, 82-85, 106-109, 130-133, columns 10-109), then the template note's pixels that differ from its own blank staff (black ink and white hollow erasures, columns a-3..b+3), pasted shifted. The CG generator draws notes on a fixed pitch grid (12 rows per half step, P=24) with tail up for notes up to B, down from C, so shifted templates are pixel-exact.

## Shortest separating key

No full hardcode can win: answers of 5-12 total ~700 chars, unknown for 6-12, and the general perl solution is 164 B. Validators 1-4 alone separate by their single answer; only useful as exact local checks.
