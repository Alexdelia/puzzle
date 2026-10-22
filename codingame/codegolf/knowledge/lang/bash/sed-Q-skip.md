# sed Q swallows the pending input chunk

Verified 2026-10-22 (shadows-of-the-knight-episode-1, OCaml `Sys.command"bash -c '...'"`, CG 100%).

`read L U;sed Q;` skips every remaining line already in the pipe: GNU sed reads a buffered block (all bytes available), `Q` quits without printing. 4 bytes under `sed 3q>&2`. It skips what is in the pipe at that moment, not a line count: good for init lines sent in one burst before the first answer, wrong when the count matters across turns.
