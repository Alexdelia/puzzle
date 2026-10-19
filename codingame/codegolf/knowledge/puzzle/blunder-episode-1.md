# Blunder - Episode 1

pretty_id `blunder-episode-1-codesize`, io.

## Rules (as referee, verified against all 12 CG tests)

- line 1 `L C` (rows first), then L map rows of exactly C chars; test 12 is `30 15`, so L != C happens
- each step: if next cell in current dir is `#` (or `X` without breaker), pick first free dir from priority list, restart from list head each time (`SENW`, inverted `WNES`); then move, print dir name
- on arrival: `$` stop; `X` in breaker mode becomes blank for good; `SENW` letters set dir; `I` toggles inversion; `B` toggles breaker; `T` jumps to the other `T`, dir and breaker kept, landing `T` does not fire again
- `@` cell is blank after start
- loop: print only `LOOP`, none of the moves; exact detection = repeat of (pos, dir, inverted, breaker, destroyed-X count): destroyed count only grows, so it stands for the map
- io limit 6000 ms per test (shared probe, `knowledge/lang/_shared/io-time-limit.md`)

## Tests

- 12 CG tests, max size 30x15; only test 11 is `LOOP`
- ignoring the inverter fails tests 07, 10, 11, 12 (12 also loops if priorities stay `SENW`)
- validators 11/12 are 100x100 serpentines (~4850 steps before the loop in 12): step-cap loop detection must allow that many steps; hidden CG validators unknown

## Input quirks

- final newline present on every test, no CR, no trailing spaces past C

## Shortest known

- none yet; plain Python reference 1419 chars at 100% on CG
