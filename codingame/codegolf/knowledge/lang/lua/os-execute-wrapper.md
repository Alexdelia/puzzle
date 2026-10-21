# os.execute wrapper

Verified 2026-10-21 on mayan-calculation (local 21/21, CG accepted): `os.execute"perl ..."` runs the perl solution with inherited stdin/stdout, 13 bytes of wrapper. Delimiter `"` beats `[[ ]]` (14) if the core has no `"`: `join"",` -> `join$,,` (same length, `$,` undef = empty).

Surface (2026-10-21, CG accepted 100%): `os.execute"perl -0nE'...'"` with a `\n` inside the Lua string (real newline in the perl regex, fine). `-0nE` (10) beats `-ln0` + shebang once `say` replaces `print`. Cell = 12 + 10 + core + 2. `say@k>>2` and `substr(...)=~O` (bareword regex) each save a byte over `~-@k/4` and `eq O`.
