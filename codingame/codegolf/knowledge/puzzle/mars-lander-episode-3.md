# Mars Lander - Episode 3

pretty_id `mars-lander-episode-3-codesize`, interactive. 2 tests (01 cave correct side, 02 cave wrong side), 2 validators.

## Rules (as referee)

- init: `surfaceN`, then `surfaceN` lines `landX landY`. Each turn: `X Y hSpeed vSpeed fuel rotate power`, answer `rotate power`
- rotate clamped to previous +-15, power to previous +-1, applied before physics
- fuel -= power (power applied in full even if fuel < power, fuel shown 0); at fuel 0 power drops to 0 at once, no ramp
- physics in doubles: `ax = -p sin(a)`, `ay = p cos(a) - 3.711`, `x += hs + ax/2`, `hs += ax` (same for y); shown values round half away from zero
- answer rotate outside -90..90 or power outside 0..4: loss
- touchdown = segment old->new position crosses the surface. Flat ground + rotate 0 + |hs| <= 20 + |vs| <= 40: win, else crash. Leaving 0..7000 x 0..3000: loss
- landing speed check on exact doubles (UNVERIFIED: rounded vs exact never probed at the limit)
- test 01: start (6500,2600) hs -20 fuel 1000 rotate 45, flat 2200..3200 y 150 under an overhang. test 02: start (6500,2000) fuel 1200, flat 3700..4700 y 220 under a ridge, must climb over the ridge top (y 2100) first

## Timing (probed on CG 2026-10-19)

- first turn: 1000 ms (0.95 s pass, 1.05 s timeout)
- next turns: 150 ms (0.14 s pass, 0.16 s timeout); statement says 100 ms, wrong

## Traps

- validator surfaces differ from test surfaces: a route chosen by `surface[0] == (0,450)` / `(0,1800)` failed one validator (CG 50%). Choosing by landing zone position (flat segment right end < 3500 or not) passed both
- crash frame in the CG log shows the interpolated hit point; Y can differ by 1 from a naive port, irrelevant to the solution (no input after the crash)
- test 02 fuel is tight: climbing to y 2550 left too little for a slow controller; y 2350 is enough over the ridge
- full thrust upright only gives +0.289 m/s2 upward: any tilt while falling loses height, vertical speed must be held near 0 during transfers
- a waypoint passed at a distance (momentum) must be dropped, not chased back
