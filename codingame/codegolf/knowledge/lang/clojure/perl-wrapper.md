# Clojure: perl wrapper

`(ns Solution(:gen-class))(.start(.inheritIO(ProcessBuilder. ["perl" "-ape" "core"])))` at top level, no -main: local 21/21 on mayan-calculation (wrapper 87 B). Avoid `"` in core (`join$,,` instead of `join"",`). CG confirmed 262 rank 1 (mayan-calculation).
