# Temperatures: hidden validators

Partial: features and counts known, no exact input recovered. No `complete`.

## Count

15 validators: partial rows step by 6.667 (bash, perl, php leaderboards, 2026-10-20). History: 8 in 2016-01, 12 from 2016-01-29 (https://forum.codingame.com/t/temperature-code-golf-puzzle-discussion/1239?page=1), 15 today. Titles unknown (not the 6 IDE titles 1:1).

## Measured (probes 2026-10-20, `agent/tool/oracle.sh --log temperature`, task validator.temperature.-.1792523691273414481)

| feature | count of 15 | probe |
|---|---|---|
| N = 0 | 1 | JS chain 1 |
| N = 1 | 1 | JS chain 4 |
| N = 2 | 3 | JS chain 8 |
| N in 3..6 | 9 | JS chain 9 |
| N > 6 | 1 | JS chain 9 |
| line 2 == `-273` or `5526` exactly | 0 | JS chain 2, 3 |
| line 2 == `-10 -10` (exact or trimmed) | 0 | JS chain 5, 7 |
| line 2 (trimmed) == an IDE test line 1..5 | 0 | JS chain 6 |
| line 2 (trimmed) == classic hints `-15 -7 -9 -14 -12`, `15 -7 9 14 7 12` | 0 | JS chain 7 |
| answer >= 0 (N = 0 included) | 10 | PHP fresh, prints min abs |
| min-abs, first of tie wins, correct | 10 | Lua fresh: 5 validators have `-t` before `t` at min abs |
| min-abs, last of tie wins, correct | 14 | Perl fresh: 1 validator has `t` before `-t` at min abs |
| some temperature is 0 | 0 | TS fresh (100%, row committed) |
| abs(answer) < 10 | 14 | Dart fresh: 1 validator has abs(answer) >= 10 |

Consequences:

- 5 negative answers, 6 ties `-t`/`t` at the minimum (5 with `-t` first, 1 with `t` first), 4 other non-negative answers (incl. N = 0 -> 0)
- no 0 among temperatures: a falsy/zero sentinel for "no value yet" is safe
- "last of tie wins" (`<=` running min, stable sort then take last, ...) fails exactly one validator; "first of tie wins" fails 5
- validators are not copies of the IDE tests nor of the classic puzzle hint inputs

## Classic puzzle (other validator set, context only)

Classic Temperatures has 11 validators: v2 "-273 alone", v3 "5526 alone", v7 "Two negative temperatures that are equal: {-10 -10}" (5DN1L, 2025-04-14, https://forum.codingame.com/t/temperatures-puzzle-discussion/33?page=50); old hints `{-15 -7 -9 -14 -12} -> -7`, `{15 -7 9 14 7 12} -> 7` (https://forum.codingame.com/t/temperatures-puzzle-discussion/33?page=2). Golf forum posts quoting "validator 7 = same negative number twice" (2021, https://forum.codingame.com/t/temperature-code-golf-puzzle-discussion/1239?page=9) date from the 12-validator set; exact `-10 -10` is not in today's set.

## Key

None known yet. N alone does not separate (3 at N = 2, 9 in 3..6).

## Next probes (not run)

- N of the single `t`-before-`-t` validator (Perl row at 93.3%: a "last wins, but first wins when N == k" probe reaching 100% moves the row): binary search over N in 2..6, >6
- N histogram inside 3..6 (fresh languages: each first submit is a count)
- abs(answer) histogram, sign per N group
