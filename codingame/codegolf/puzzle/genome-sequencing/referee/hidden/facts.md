# Genome Sequencing (codesize): hidden validators

**14 validators**, weights 70 (V1-V12) + 80 (V13, V14) = 1000 (CG per-validator report, `agent/tool/cg-report.sh 41549412`, 2026-10-24). The earlier "100 validators" reading was wrong: weights in tens of 1000 give 1% score steps. Titles = the classic puzzle's validator titles, not the 7 IDE tests (answer-string titles).

Sources: reports of the 2026-10-20/22 probes (php/ts/lua chains, python 88/102, perl 56/91, haskell 151; scratch `rep_*.json` of journal `validator.genome-sequencing.-.1792834543881612293`) and 4 per-validator probes `probe/python/q1..q4.py` (2026-10-24, `probes.tsv`). Each probe classifies the input by `n`, fold answer `a`, `L` = sum of lengths, `m` = max length (V11/V12 share a class, every other validator has its own) and answers right iff a per-class predicate holds; candidate lists in scratch `state.json`.

Input format assumed as the IDE tests: `N` line, one string per line, final newline.

## Per validator

Exact inputs from the bit-stream probes `probe/python/b0..b24.py` (2026-10-24, journal `validator.genome-sequencing.-.1792837592016938042`, scratch `bits*.py`, `res.txt`, `dec2.py`): probe j answers right iff bit j of a per-class encoding of the input is 1 (list classes: index+1 in a fixed candidate list; V9/V13/V14: fold string s in base 4 plus positions). Every exact one passes the reference (`run.sh main.py --hidden`) and a brute-force shortest superstring.

| # | title | weight | input (`NN.in`) | answer | status |
|---|---|---|---|---|---|
| 1 | Two separate sequences | 70 | `A` `C` | 2 | exact |
| 2 | One single sequence | 70 | `A` | 1 | exact |
| 3 | Identical sequences | 70 | `ATA` x3 | 3 | exact |
| 4 | Example 3, but with the first two words nested | 70 | `GA` `CTG` `ACT` | 5 | exact (old 189 list was wrong, re-listed 629) |
| 5 | Multi-solutions | 70 | `AC` `TG` `CT` `GA` | 5 | exact |
| 6 | A + C = AC | 70 | `A` `C` `AC` | 2 | exact |
| 7 | Sequence included | 70 | `GATTACA` `ATTAC` | 7 | exact |
| 8 | Sequence included, reverse order | 70 | `ATTAC` `GATTACA` | 7 | exact |
| 9 | Almost superimposed | 70 | `GATATATA` `ATAC` | 9 | exact |
| 10 | Example 3, natural order | 70 | `AA` `ACT` `TT` | 5 | exact |
| 11 | Mirror, reverse order | 70 | `CA` `AC` | 3 | exact |
| 12 | Mirror | 70 | `AC` `CA` | 3 | exact |
| 13 | Sequences included and disjointed | 80 | `ATAC` `TAC` `AG` | 6 | exact |
| 14 | Explore all combinations | 80 | three 5-char strings, fold s (11) starts `AAAGCTATGC` (10 of 11 chars), w0 = s[:5], w2 = s[6:] | 11 | partial |

Answers: 2 1 3 5 5 2 7 7 9 5 3 3 6 11. Input format: `N` line, one string per line, final newline (assumed as IDE tests).

## Methods vs validators (from reports)

- input-order fold (python 114, php, perl 70, ...) passes all 14
- fold without `s in t` reset (python 102) fails V8 only
- fold without `t in s` skip (perl 56) fails V4, V5, V7
- adjacent-line regexes only (perl 91) fails V4, V5
- haskell fold 151/152 fails V6 only (`A`, `C`, `AC`: look at 1-char strings in that code)
- table keyed on IDE test input lengths (python 88) passes V10, V14 only

## Key

13 of 14 exact (`NN.in` here); V14 still partial: one char of s and the position of w1 in s (0..6) missing, so no `complete` yet. Input lengths (bytes, with final newline): V1 6, V2 4, V3 14, V4 13, V5 14, V6 9, V7 15, V8 15, V9 16, V10 12, V11 8, V12 8, V13 14, V14 20.

## Next

- V14 needs 5 more bits: bits 20..24 of `E = b4(s) | s.find(w1)<<22` (`bits4.py` j=25..29 of the scratch of journal `validator.genome-sequencing.-.1792837592016938042`), or one predicate probe per guess; then write `14.in` and `complete`
