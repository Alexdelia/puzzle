# Mars Lander - Episode 3

pretty_id `mars-lander-episode-3-codesize`, leaderboard `mars-lander-episode-3-codesize`, interactive.

## Rules (as referee)

- init: `N` then `N` lines `x y` (surface polyline, x 0..6999). Each turn: `X Y hSpeed vSpeed fuel rotate power`, answer `rotate power`
- zone 7000 x 3000, gravity 3.711. rotate step <= 15 per turn, power step <= 1 per turn
- physics in doubles: `ax=-p*sin(a)`, `ay=p*cos(a)-3.711`, `x+=hs+ax/2`, `hs+=ax` (same for y). Input values rounded half away from zero, no `-0`
- fuel: decreases by applied power; fuel 1 with power 3 still applies power 3 (fuel shown 0); at fuel 0 power drops to 0 at once (no ramp)
- answer rotate outside -90..90 or power outside 0..4: loss
- landing: movement segment crosses flat segment with rotate 0, |hs|<=20, |vs|<=40 (doubles; rounded vs exact at the boundary UNVERIFIED). Crossing a non-flat segment or leaving the zone: loss
- tests: 01 "cave, correct side" (6500 2600 -20 0 1000 45 0), 02 "cave, wrong side" (6500 2000 0 0 1200 0 0). Full fixtures in `referee/tests/`

## Timing (probed on CG 2026-10-19)

- first turn 1000 ms (0.95 s pass, 1.05 s timeout), next turns 150 ms (0.14 pass, 0.16 timeout); statement says 100 ms, wrong

## Validator trap

- 2 hidden validators, NOT equal to the tests. A route hard-coded per map (waypoints keyed on landing zone x) passed both CG tests and scored 50% on submit
- a generic planner (visibility graph over terrain vertex offsets + waypoint controller) scored 100%: plan from the surface, do not hard-code coordinates
- local `referee/validators/` hold only perturbed starts on the test maps: they do not catch overfit to the map shape. Stress with jittered terrain (+-80) and mirrored maps before submitting

## Controller traps (verified locally)

- tilt 15 at power 4 gives only +0.15 m/s2 vertical: climbing with a horizontal target sinks the lander into the peak at (4900,2050) on test 02
- capping descent speed below the leg slope makes the lander fly above a steep leg: hits the overhang tip (3150,1550) on test 01
- rotate must reach 0 one turn before touchdown (rate 15/turn)

## Shortest known

- ours: Python 1952 (generic planner, not golfed hard). Top Python 57 (Waffle3z): far shorter approach exists, unknown
