# ascii-art hidden validators (CG `ascii-art-codesize`)

Complete: all 7 validators recovered exactly (task `validator.ascii-art.-.1792834543677482909`, 2026-10-24). Files `01..07.in/.out` here; perl, python and lua solutions pass `run.sh <solution> --lang <lang> --hidden` 13/13.

## Count and weights

7 validators, weights 300/200/100/100/100/100/100 (sum 1000). Source: CG per-validator report (`agent/tool/cg-report.sh 41541589`, and every later probe). The old reading "10 validators" (score grain 10 %) was the weight grain, not a count.

## Per validator

All inputs are `L`, `H`, `T`, then the font rows of an IDE test, verbatim (fonts: lua probe, font hash in {test 1 font, test 5 font} passed all 7, submission 41537404).

| # | title | weight | L H font | `T` | source | confidence |
|---|---|---|---|---|---|---|
| 1 | Test only one letter : G | 300 | 4 5 test-1 `#` font | `G` | python probe `exact1.py` pass; python 58 single-uppercase `L=4` probe (submission 41536725) passed only this one | exact |
| 2 | Test HELLOWORLD | 200 | 4 5 `#` | `HELLOWORLD` | `exact1.py`; go probe `T==HELLOWORLD` (41537627) passed only this one | exact |
| 3 | Test HelloWorld | 100 | 4 5 `#` | `HelloWorld` | `exact1.py` | exact |
| 4 | Test AB!!CD | 100 | 4 5 `#` | `AB!!CD` | `exact1.py` | exact |
| 5 | Test Lorem ipsum dolor sit amet,... | 100 | 4 5 `#` | `Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse lectus tortor, dignissim sit amet, adipiscing nec, ultricies sed, dolor.` (156 chars) | probes `lorem2..7.py` (prefix, length > 60, `. ` at 55, `Sed non risus` at 57, exact equality) | exact |
| 6 | Test full alphabet in CAPS WMADXESFGIJVTHKNOBPLYQRUZC | 100 | 4 5 `#` | `WMADXESFGIJVTHKNOBPLYQRUZC` | `exact1.py` | exact |
| 7 | Test HELLO with another ASCII representation | 100 | 20 11 test-5 font | `HELLO` | `exact1.py`; cpp probe `L==20` (41537718) passed only this one | exact |

Every probe line is in `puzzle/ascii-art/probe/probes.tsv`.

## Features over all 7 (what a golfed method must handle)

- non-letter chars in `T`: only ` ` `!` `,` `.` (validators 4 and 5), all below `A` (codes 32, 33, 44, 46). No `@ [ \` {` and no digit in any validator (`@` is only in IDE test 4, which CG does not score)
- lowercase: validators 3 and 5
- `L=4 H=5` except validator 7 (`L=20 H=11`)
- `T` lengths 1, 10, 10, 6, 156, 26, 5; no leading or trailing space

## Shortest separating key

`T[0]` separates all but 2, 3, 7 (`G H H A L W H`); `T[1]` then splits them (`E` for 2 and 7, `e` for 3), `L` (or `len(T)`: 10 vs 5) splits 2 and 7. `len(T)` alone fails (2 and 3 both 10). The answers are the H rows of the glyph concatenation; 2 and 3 have the same answer.
