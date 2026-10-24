# `#!ruby -pl` / `-nl` shebang loops on CG (interactive)

CG runs Ruby solutions as `ruby main.rb`, and Ruby itself honours switches on a
`#!ruby ...` first line (verified locally 3.1.2 and on CG: the 91/77 B
power-of-thor rows rely on `-p`/`-n`/`-l`).

- `-p` makes the body run per input line and auto-prints `$_`; `-l` chomps `$_`
  and sets `$\` to `"\n"`, so `$_=expr` needs no explicit newline and no print call.
- `$.` counts input lines but is shifted under `-n`/`-p`: the first input line
  gets `$.=2` (the shebang line is counted). So `$.` on the energy line of turn t
  is t+1, and a loop iteration k reads the *previous* turn's energy line
  (turn-1 lag) while emitting move k.
- `$>.sync=1` (8 B) is still required on CG: a 67 B `-pl` variant without it was
  probed 2026-10-24 and did NOT reach 100% (row stayed at 77), so CG pipes stdout
  with full buffering, not a pty.
- live example: power-of-thor ruby 77, CG 100% (2026-10-24):
  `#!ruby -pl` + `$>.sync=1` + `e=$_.to_i;h||="WEN"[e%3]` +
  `$_=($.>18&&46>e&&3<e&&?S||"")+h`
