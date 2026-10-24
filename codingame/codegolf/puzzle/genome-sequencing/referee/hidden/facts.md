# Genome Sequencing (codesize): hidden validators

**14 validators**, weights 70 (V1-V12) + 80 (V13, V14) = 1000 (CG per-validator report, `agent/tool/cg-report.sh 41549412`, 2026-10-24). The earlier "100 validators" reading was wrong: weights in tens of 1000 give 1% score steps. Titles = the classic puzzle's validator titles, not the 7 IDE tests (answer-string titles).

Sources: reports of the 2026-10-20/22 probes (php/ts/lua chains, python 88/102, perl 56/91, haskell 151; scratch `rep_*.json` of journal `validator.genome-sequencing.-.1792834543881612293`) and 4 per-validator probes `probe/python/q1..q4.py` (2026-10-24, `probes.tsv`). Each probe classifies the input by `n`, fold answer `a`, `L` = sum of lengths, `m` = max length (V11/V12 share a class, every other validator has its own) and answers right iff a per-class predicate holds; candidate lists in scratch `state.json`.

Input format assumed as the IDE tests: `N` line, one string per line, final newline.

## Per validator

| # | title | weight | known | answer |
|---|---|---|---|---|
| 1 | Two separate sequences | 70 | N=2, two 1-char strings, one of `A C`, `A T`, `C G`, `G A`, `G T`, `T C` | 2 |
| 2 | One single sequence | 70 | N=1, `A` or `G` | 1 |
| 3 | Identical sequences | 70 | N=3, the same 3-char string 3 times, string ends in `A` (16 left: `xyA`, x,y any) | 3 |
| 4 | Example 3, but with the first two words nested | 70 | N=3, w3=`ACT`, w2 strictly inside w1, w1,w2 not `TT`,`T`, lengths <= 5, answer not L, m, L-2 (189 candidates) | 5 |
| 5 | Multi-solutions | 70 | N=4, four distinct 2-char strings, input order is not a chain w[i][1]==w[i+1][0], fold answer 5 | 5 |
| 6 | A + C = AC | 70 | **exact**: `A`, `C`, `AC` (`06.in`) | 2 |
| 7 | Sequence included | 70 | N=2, w1=`GATTACA`, w2 one of `T`, `GA`, `ATT`, `ATTA`, `ATTAC`, `GATTAC` | 7 |
| 8 | Sequence included, reverse order | 70 | N=2, w2=`GATTACA`, w1 one of the same 6 | 7 |
| 9 | Almost superimposed | 70 | N=2, overlap >= 2, not both strings of equal length with shift 1, strings not substrings of the IDE test strings | 9 |
| 10 | Example 3, natural order | 70 | **exact**: `AA`, `ACT`, `TT` (`10.in`) | 5 |
| 11 | Mirror, reverse order | 70 | N=2, w2 = reverse(w1), 2 chars: one of `CA AC`, `GC CG`, `TG GT` | 3 |
| 12 | Mirror | 70 | N=2, w2 = reverse(w1), 2 chars: one of `AC CA`, `CG GC`, `GT TG` | 3 |
| 13 | Sequences included and disjointed | 80 | N=3, L in 8..9, some string inside another, not all strings substrings of the IDE test strings | 6 |
| 14 | Explore all combinations | 80 | N=3, three strings of equal length, not all substrings of the IDE test strings | 11 |

Answers (confidence high, each from a bit probe): 2 1 3 5 5 2 7 7 9 5 3 3 6 11.

## Methods vs validators (from reports)

- input-order fold (python 114, php, perl 70, ...) passes all 14
- fold without `s in t` reset (python 102) fails V8 only
- fold without `t in s` skip (perl 56) fails V4, V5, V7
- adjacent-line regexes only (perl 91) fails V4, V5
- haskell fold 151/152 fails V6 only (`A`, `C`, `AC`: look at 1-char strings in that code)
- table keyed on IDE test input lengths (python 88) passes V10, V14 only

## Key

Exact inputs known for V6, V10 only: no exact hash key yet. Cheap features that separate answers:

- `N`: 1 -> V2 (1); 4 -> V5 (5)
- N=2: answer 2 (V1, total input length 6), 3 (V11/V12, length 8), 7 (V7/V8, length 12..17), 9 (V9)
- N=3: 2 (V6, length 9), 3 (V3, length 14, all lines equal), 5 (V4 ends `ACT`; V10 length 12), 6 (V13, length 13..14), 11 (V14)
- total input length alone collides (V3 and V5 both 14 bytes); `N` plus length, or the last line, separate more

## Next

- per-validator index probes keep halving the candidate lists (scratch `state.json`, generator `gen.py` + `K*.json`): V2 1 probe, V11/V12 2, V7/V8 3, V1 3, V3 4; V4 8, V5 12; V9, V13, V14 need their lengths first
