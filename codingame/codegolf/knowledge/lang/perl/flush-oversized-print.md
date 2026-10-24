# Flush buffered output with one oversized print (no `$|`)

An interactive one-shot (plan printed at turn 1) needs stdout flushed. Instead of
`$|=` (3 bytes), make the *last* print oversized: the buffered turn-1 answer is
written before the big string (C stdio flushes when the request exceeds the
buffer), and the big write then blocks on the full 64 KB pipe
(knowledge/lang/_shared/pipe-block-keepalive.md), keeping the process alive.

- `$|=<>=~/ /;...;print"WAIT\n"x99` -> `<>=~/ /;...;print"WAIT\n"x1e5`: `$|=` -3,
  repeat `x99` -> `x1e5` +1, net **-2**
- verified the-fall-episode-3 perl 137 -> 135, CG 100% all 4 validators (probe
  2026-10-24, `puzzle/the-fall-episode-3/probe/probes.tsv`); local referee
  `--hidden` passes
- the string is built in memory: `1e5` repeats of 5 bytes = 500 KB, keep it low MB
- use only when the last answer is harmless repeated (`WAIT`, a landing no-op, ...)
- per-turn interactive cells get no gain when the flush is already fused with
  the header store (`$|||=@a=@F;`, knowledge/lang/perl/array-guard-autoflush.md):
  the trick needs the whole move plan printed at turn 1, and a multi-line plan
  pays one newline per move (e.g. `"S$h\n"x($F[1]-$F[3])."$h\n"x1e5`).
  power-of-thor perl: the flush-free plan-print is exactly 61 B, the same as
  the per-turn row, so net 0, not -2 (local hidden 4/4, 2026-10-24, unverified
  on CG).

Measured trap (shadows-of-the-knight-episode-1, 2026-10-24): one *single* line
per turn cannot be flushed by oversizing it. The 92 B solution without `$|=`
(89 B) times out turn 1, and padding the answer line with `" "x1e4` (the
referee trims trailing spaces, so the padding is legal) still times out: perl's
8192 B buffer flushes the first 8192 B, the line's newline stays in the
unwritten tail, and the program then blocks reading the next direction. Padding
after the newline flushes the answer but the leftover spaces become a partial
line that merges with the next answer (leading spaces fail). One oversized print
only delivers whole lines that are already complete inside the flushed prefix,
i.e. multi-line plans. Per-turn output keeps `$|=` (fused, 3 B).

