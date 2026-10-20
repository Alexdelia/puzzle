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
- short method NOT FOUND yet (2 search sessions). Tops: Perl 37, Ruby 48, bash 51, JS 52, Python 57: a 10-20 char formula over one or two input fields, likely power 4 constant or near
- facts that bound the method (verified on CG): answers queue, one per input line, so answering every line (surface lines too) makes turn t use line t (lag N+1: 23 on test 01, 19 on test 02); Perl needs `$|=1;` (block-buffered stdout, timeout turn 1 without it); Y>=3000 is a loss ("lost in outer space": `0 4` forever dies frame 107 on test 02)
- length hint: `$|=1;print int($_/D-C)," 4\n"while<>` is 37-38 chars, and Perl `$_` numeric = first field (X, surface x, or N), so rotate=f(X) with power 4 fits the Perl top exactly. The sim says no such constant pair lands both tests (below)
- rejected in the sim (`sim.c` in scratch of journals ...1792440302536411339 and ...1792441435385604734; lag on and off; must land tests 01 and 02):
  - rotate trunc((X-C)/D), every C 0..6999, D 1..400, power 3 or 4 const (lag on); floor variant, C step 5 (lag on and off)
  - rotate X/D-C, X%M-C, (X/D)%M-C, C-(X/D)%M, power 3|4 const
  - rotate (aX+bY-C)/D, a,b in -2..2; power const 4, linear in X,Y, or X,Y threshold 3|4 (random, ~1e8 tries)
  - rotate (a hs+b vs+c Y/100-C)/D, power 4 or vs/Y threshold 3|4 (random)
  - prior session: (x+k f-C)/D+(a h+b v)/E, power threshold on v and y (80M random)
- untested next: several answers per line (`print"..."x k` bursts = open-loop schedule keyed on surface lines), Perl regex captures surviving failed matches (surface vs turn lines), per-word answers (`$/=$"`), rotate keyed on fuel or `$.`. Or the sim misses a CG rule: run the best sim near-miss on CG before trusting sim rejections
