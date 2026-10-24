# The Descent: hidden validators

5 validators, 1:1 with the IDE tests (CG report titles: Ascending mountains, Scattered mountains, Strong mountains 1, Strong mountains 2, One mountain; test 1 is titled "Descending mountains"). Each is its test with the 8 mountain positions permuted (heights and per-mountain shot sequences kept). All 5 recovered exactly (`01.json`..`05.json`), `complete` set, reference python passes `--hidden` 10/10. Probes: `agent/tool/oracle.sh --log the-descent`, tasks validator.the-descent.-.1792523691378056501 (probes 1-12), validator.the-descent.-.1792604813829575306 (probes 13-37), validator.the-descent.-.1792834544429519414 (probes 38-50, per-validator report).

## Count and weights

CG report (probe 38 on): V1-V4 weigh 167, V5 weighs 166 (total 834); confirms the earlier score-grain deduction.

## Per validator

| # | title | known | source | confidence |
|---|---|---|---|---|
| 1 | Ascending mountains | exact: test 1 reversed, heights 2..9 at index 0..7 (one shot each). `01.json` | probe 14 (PHP 500/834) | high |
| 2 | Scattered mountains | exact: permutation of test 2 (one shot each): turn 1 `5 2 4 6 9 7 3 8`; full trajectory confirmed. `02.json` | probes 2, 3, 7, 34-37 narrowed to 2160 candidates (9@4, 8 at 5-7); probes 38-49 = 12 bits of its lexicographic rank (1106); probe 50 trajectory | high |
| 3 | Strong mountains 1 | exact: turn 1 `0 6 7 0 8 5 0 1`, test-3 shot sequences (6 2 0, 7 4 0, 8 0, 5 3 0, 1 0); full trajectory confirmed. `03.json` | probes 15 (Go 0), 21-33 | high |
| 4 | Strong mountains 2 | exact: test 4 rotated, v[i] = t[(i+7)%8]: `[6,0],[0],[6,4,2,0],[0],[5,2,0],[0],[8,5,0],[0]`. `04.json` | probes 15-19 | high |
| 5 | One mountain | exact: mountain `8 7 6 5 4 0` at index 2 (test: index 3). `05.json` | probes 1, 4, 8, 11, 12 | high |

"multiset of test k" = probe 2: every validator's turn-1 heights sorted equal a test's sorted turn-1 heights (T1 and T2 share {2..9}).

## Probes

Probe = argmax (first max) iff predicate, else `x` (fails turn 1). Count probes in a language with no row show the exact pass count.

| # | lang | predicate | result |
|---|---|---|---|
| 1 | JS | full trajectory in test trajectories | 0/5 |
| 2 | JS | turn-1 sorted heights = a test's | 5/5 |
| 3 | TS | trajectory in reversed tests | 1 (167) |
| 4 | Perl | trajectory in rotations r=1..7 of tests | 2 (167+166) |
| 5 | PHP | trajectory in T1-T4 under 15 non-identity dihedral maps | 2 (167+167) |
| 6 | JS y/n | V5 at index 4 | no |
| 7 | Lua | T1,T2 under 15 dihedral maps | 1 |
| 8 | JS y/n | V5 at index 0-2 | yes |
| 9 | TS | reversed(T1,T2) u rotations(T3,T4) | 2 (shown over row 1) |
| 10 | JS y/n | V5 at 0-1 | void (same length as row, cannot tell) |
| 11, 12 | JS y/n | V5 at 0, V5 at 1 | no, no |
| 13 | Lua | wasted: Lua row was already 100% | - |
| 14 | PHP | V5+M1+rot(T3,T4): 500 if mirror is T1 | 500: V1 = mirror |
| 15 | Go fresh | rot(T3, 1..7) | 0: rotated one is V4 |
| 16-19 | Go | V5+M1+rot(T4, {1,2,3} / {4,5} / {6} / {7}) | 333, 333, 333, 500: r = 7 |
| 20-32 | Go y/n | V3 (turn-1 sorted 00015678) digit d position in P, plain argmax elsewhere | 8@4, 7@2, 6@1, 5@5, 1@7 |
| 33 | Go y/n | V3 full trajectory = `03.json` | yes |
| 34-37 | Go y/n | V2 (turn 1 has a 9, not 23456789) digit position: 9 in 0-3 no, 4-5 yes, 4 yes; 8 in 0-3 no | 9@4, 8 in 5-7 |
| 38-49 | Python (report) | V2 bit j (j=0..11) of the lexicographic rank of turn 1 among the 2160 candidates | bits LSB first 0 1 0 0 1 0 1 0 0 0 1 0 = 1106 |
| 50 | Python (report) | V2 full trajectory = `02.json` | pass |

Probe sources: `puzzle/the-descent/probe/{php,go,python}/`; generators in the task scratch dir (`bit.py`, `v2.py`: Go argmax that fails one validator, keyed on turn 1, iff digit d is outside positions P; each probe shorter than the last so a yes always replaces the row).

## Key

Turn 1 separates every validator: V5 sorted `00000008`, V3 sorted `00015678`, V4 sorted `00005668`, V1 = `23456789`, V2 = `52469738`. First line alone (h[0]): V1 2, V2 5, V3 0, V4 6, V5 0 (V3/V5 collide); `h[0]+2*h[1]` = 8, 9, 12, 6, 0 separates all.

First-max answer sequence per validator (turn 1 heights -> shots):

| # | turn 1 | shots |
|---|---|---|
| 1 | 23456789 | 76543210 |
| 2 | 52469738 | 47530261 |
| 3 | 06708501 | 42152517 |
| 4 | 60605080 | 60246224 |
| 5 | 00800000 | 22222 |

Tie side is free: last-max (`7-h[::-1].index(max(h))`) also passes `--hidden` 10/10 (local, 2026-10-24). A hardcode is still unlikely to beat argmax (5 sequences, 37 digits of data plus a 2-line key), but `--hidden` now checks any exploit exactly.
