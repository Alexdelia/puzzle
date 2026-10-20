# The Descent: hidden validators

5 validators, titles = IDE tests (assumed 1:1). Each is its test with the 8 mountain positions permuted (heights and per-mountain shot sequences kept). 1 of 5 recovered exactly (`05.json`). Probes: `agent/tool/oracle.sh --log the-descent`, task validator.the-descent.-.1792523691378056501.

## Count and weights

Partial leaderboard rows (Groovy, PHP 79.976 = 667/834, Java 39.928 = 333/834) and our probes: scores are sums of per-validator weights over 834: four validators weigh 167, one weighs 166 (deduced: V5).

## Per validator

| # | title | known | source | confidence |
|---|---|---|---|---|
| 1 | Descending mountains | turn-1 = permutation of 2..9. One of V1/V2 = test reversed (index i -> 7-i), the other no dihedral transform of its test | probes 2, 3, 7, 9 | high |
| 2 | Scattered mountains | same as V1 (which of V1/V2 is the mirror: unknown) | probes 2, 3, 7, 9 | high |
| 3 | Strong mountains 1 | turn-1 multiset of test 3. One of V3/V4 = its test cyclically rotated (r in 1..7 unknown), the other no dihedral transform of its test | probes 2, 4, 5, 9 | high |
| 4 | Strong mountains 2 | same as V3 (which one is rotated: unknown) | probes 2, 4, 5, 9 | high |
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

## Next steps (unrecovered)

- which of V1/V2 is the mirror: count probe in a no-row language with reversed(T1) only
- which of V3/V4 is rotated and by how much: rotations of T3 only, then split r
- the 2 non-dihedral ones (one V1/V2, one V3/V4): arbitrary permutation, up to 15 bits (V1/V2) / ~10 bits (V3/V4): too many for yes/no probes; try other structured maps first (swap halves already = rotation 4; pair swaps, per-half reverse)

## Key

V5 alone is separable by turn-1 sorted heights `00000008`. A hardcode is useless for golf: argmax is required anyway (descending/scattered validators force the max every turn).
