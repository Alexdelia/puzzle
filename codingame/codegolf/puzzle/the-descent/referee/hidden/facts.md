# The Descent: hidden validators

5 validators, titles = IDE tests (assumed 1:1). Each is its test with the 8 mountain positions permuted (heights and per-mountain shot sequences kept). 4 of 5 recovered exactly (`01.json`, `03.json`, `04.json`, `05.json`); V2 partial. Probes: `agent/tool/oracle.sh --log the-descent`, tasks validator.the-descent.-.1792523691378056501 (probes 1-12) and validator.the-descent.-.1792604813829575306 (probes 13-37).

## Count and weights

Partial leaderboard rows (Groovy, PHP 79.976 = 667/834, Java 39.928 = 333/834) and our probes: scores are sums of per-validator weights over 834: four validators weigh 167, one weighs 166 (deduced: V5).

## Per validator

| # | title | known | source | confidence |
|---|---|---|---|---|
| 1 | Descending mountains | exact: test 1 reversed, heights 2..9 at index 0..7 (one shot each). `01.json` | probe 14 (PHP 500/834) | high |
| 2 | Scattered mountains | partial: permutation of test 2 (heights 9 8 7 3 6 5 2 4, one shot each), not a dihedral map of it. turn 1: `9` at index 4, `8` at index 5, 6 or 7. 7! / 7 * 3 = 2160 candidates left | probes 2, 3, 7, 34-37 | high |
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

Probe sources: `puzzle/the-descent/probe/{php,go}/`; generators in the task scratch dir (`bit.py`, `v2.py`: Go argmax that fails one validator, keyed on turn 1, iff digit d is outside positions P; each probe shorter than the last so a yes always replaces the row).

## Next steps (unrecovered)

- V2 only: about 11 bits left (positions of 8 in {5,6,7}, then 7, 6, 5, 4, 3, 2 among the rest). Same Go y/n probes, `v2.py <digit> <positions> <length>`, length under the Go row (618 on 2026-10-21). V2 keeps the test-2 sequences (one shot each), so turn 1 fixes it; check the full trajectory at the end like probe 33
- once V2 is written: reference `--hidden`, then create `complete`

## Key

Turn 1 separates every validator: V5 sorted `00000008`, V3 sorted `00015678`, V4 sorted `00005668`, V1 = `23456789`, V2 = the other one with a 9. A hardcode is useless for golf: argmax is required anyway (descending/scattered validators force the max every turn).
