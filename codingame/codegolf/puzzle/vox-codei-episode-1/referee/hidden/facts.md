# Vox Codei - Episode 1: hidden validators

11 validators, titles = IDE test titles, score weighted by the title % (10,10,10,10,10,10,10,9,8,7,6; CG js row read 63.0 and 71.0). Every validator = its IDE test with the SAME rounds and bombs and the grid mirrored (dims unchanged, no translation, no transpose). All 11 recovered exactly, confidence: CG-verified (js probe 2278 = 100% passes only on the exact grids below; perl/js probes in `agent/state/oracle/vox-codei-episode-1.tsv`).

Transform: `hflip` = reverse each row, `vflip` = reverse row order, `rot180` = both.

| # | title | w x h | rounds | bombs | nodes | transform of test | source |
|---|---|---|---|---|---|---|---|
| 1 | One node, one bomb | 4x3 | 15 | 1 | 1 | hflip (node at x2 y1) | js probe 5 (no transpose) + perl probe 1 (not identity) |
| 2 | 3 nodes, 3 bombs | 4x3 | 15 | 3 | 3 | hflip (= rot180) | same |
| 3 | 9 nodes, 9 bombs | 12x9 | 15 | 9 | 9 | rot180 | js probes 6, 7 |
| 4 | 4 nodes, 1 bomb | 4x3 | 4 | 1 | 4 | hflip (= rot180) | js probe 5 + perl probe 1 |
| 5 | Lot of nodes, few bombs | 8x6 | 15 | 3 | 21 | rot180 | js probe 8 (exact, 100%) |
| 6 | 4 scattered nodes, 2 bombs | 8x6 | 5 | 2 | 4 | hflip (= rot180) | js probe 5 + perl probe 1 |
| 7 | Indestructible nodes | 12x9 | 15 | 4 | 8 (16 #) | vflip (= rot180) | same |
| 8 | Foresee the future | 8x6 | 10 | 3 | 9 | rot180 | js probes 6, 7 |
| 9 | Foresee the future better | 8x6 | 10 | 2 | 9 | hflip | js probe 7 (+8) |
| 10 | Destroy CodinGame | 12x9 | 15 | 6 | 22 (4 #) | vflip | js probe 6 (+7) |
| 11 | Not so fast! | 15x12 | 15 | 4 | 8 (16 #) | vflip (= rot180) | js probe 5 + perl probe 1 |

Exact inputs: `NN.json` here (same format as `tests/`).

Probe chain (2026-10-20): perl fresh `header == IDE test` 0% -> perl `R,B equal and @/# pattern equal up to dihedral+translation` 100% -> perl `grid == one of 8 dihedral transforms of test k` 100% -> perl `all rot180` no -> js fresh `any transpose` 0% -> js weighted increments decode v10 vflip (+7), v9 hflip (+8, v3/v8 not) -> js exact 100%.

## Keys

- `(w, rounds, bombs)` separates all 11: 1 (4,15,1), 2 (4,15,3), 3 (12,15,9), 4 (4,4,1), 5 (8,15,3), 6 (8,5,2), 7 (12,15,4), 8 (8,10,3), 9 (8,10,2), 10 (12,15,6), 11 (15,15,4)
- `(rounds, bombs)` alone collides 2/5 (15,3) and 7/11 (15,4); bombs alone: 1,3,9,1,3,2,4,3,2,6,4
- index of first `@` in the rows concatenated (no separator): 6 4 14 2 0 0 50 17 9 26 37 (5/6 collide)
- IDE tests are NOT scored on submit: a hardcode only has to pass these 11 (but local `submit.sh` gate then checks tests + hidden, see `.doc/validators.md`)

## Local referee (2026-10-25)

- `src/runner.rs` `ClosedStdin<G>` ignores failed turn-input writes like CG and keeps reading buffered stdout; a player that exits without enough lines still fails `solution exited`
- JVM launchers get a 1500 ms first turn (CG frame-1 is ~1.5 s JVM uptime); all other langs 1000 ms
- all 27 `solution/*/main.*` (all at their cell `.ours` length) pass `--cg` 11/11; bash 159 x100 under load 0 fails
