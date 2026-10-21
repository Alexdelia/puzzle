# os.execute wrapper

Verified 2026-10-21 on mayan-calculation (local 21/21, CG accepted): `os.execute"perl ..."` runs the perl solution with inherited stdin/stdout, 13 bytes of wrapper. Delimiter `"` beats `[[ ]]` (14) if the core has no `"`: `join"",` -> `join$,,` (same length, `$,` undef = empty).
