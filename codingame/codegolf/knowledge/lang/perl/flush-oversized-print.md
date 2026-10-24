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
