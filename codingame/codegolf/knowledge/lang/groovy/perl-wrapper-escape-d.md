# Groovy perl wrapper: `\d` in single quotes (shadows-of-the-knight-episode-2, CG 287 #1, 2026-10-21)

`'...\d...'` is a Groovy compile error ("Unexpected input"): write `\\d` (+1 B). Perl core from `knowledge/puzzle/shadows-of-the-knight-episode-2.md` (lua section) as `new ProcessBuilder('perl','-E','<core>').inheritIO().start()`. Local runs can time out turn 1 (JVM + perl startup) under load; rerun.
