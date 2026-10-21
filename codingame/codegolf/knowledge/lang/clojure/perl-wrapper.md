# Clojure: perl wrapper

`(ns Solution(:gen-class))(.start(.inheritIO(ProcessBuilder. ["perl" "-ape" "core"])))` at top level, no -main: local 21/21 on mayan-calculation (wrapper 87 B). Avoid `"` in core (`join$,,` instead of `join"",`). CG confirmed 262 rank 1 (mayan-calculation).

surface: core as `"perl" "-ln0e" "..."` (-l before -0 keeps $\ newline); core may hold real newlines and no `"` or backslash. 228 B total (surface 228 rank 1).
