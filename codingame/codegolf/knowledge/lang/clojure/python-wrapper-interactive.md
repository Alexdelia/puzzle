# Clojure: python3 core on an interactive puzzle

Verified on CG 2026-10-21 (the-fall-episode-2, 100% at 1059 then 995, probes in `puzzle/the-fall-episode-2/probe/probes.tsv`): `python3` is on PATH in the CG Clojure box (ruby is not, see `knowledge/puzzle/music-scores.md`).

- `(ns Player)(defn -main[_](..(ProcessBuilder.["python3""-c""<core>"])inheritIO start waitFor))` = core + 86 B
- `waitFor` needed: the Bridge's -main returns at once, JVM exits, orphan child is killed (`../_shared/orphan-child-kill.md`)
- core must hold no `"` and no backslash (Clojure string); real newlines and indentation are fine
- no `-u` needed: python `input()` flushes stdout before reading
- local check: the dev shell's CG_CLOJURE may lack `clojure-run-interactive` (compiles `Solution`, "Could not locate Solution__init"); build a fresh one with `nix build --no-link --print-out-paths --impure --expr '(import ./agent/tool/lang/clojure/package.nix { pkgs = (builtins.getFlake (toString ./.)).inputs.nixpkgs.legacyPackages.x86_64-linux; })'` and export CG_CLOJURE to it. Even then the local referee counts JVM startup in the 1000 ms first turn: most tests time out on turn 1 locally (power-of-thor clojure, CG 100%, also fails locally). Verify the python core with `--lang python`, send the wrapper via `oracle.sh`
