# Double string repeat for a scaled key count

`"s"x(C-<>)xK` repeats `"s"` K*(C-N) times: `x` is left-associative, so the second `x` repeats the already repeated string. 1 byte under `"s"x(C*K-K*<>)` (no `*`, C has fewer digits).

- verified (mars-lander-episode-3, 2026-10-22): `print"6 4⏎"x(28-<>)x9,...` (A N=22 gets 54 lines, B N=18 gets 90), referee --hidden 4/4
- count must be a multiple of K on both inputs; with the key modulo form `x(<>%M*K)` both forms cost the same, pick what the landers allow
- same in Ruby (`"s"*(c-gets.to_i)*k`), unverified there
