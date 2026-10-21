# Haskell runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- GHC 8.4.3 (`System.Info`: `ghc 8.4.3 linux x86_64`, `__GLASGOW_HASKELL_PATCHLEVEL1__` 3), so base 4.11 (`<>` in Prelude); extensions newer than 8.4 (`BlockArguments` 8.6+) unavailable
- `{-# LANGUAGE CPP #-}` pragma accepted
- `getLine` on a last line without final newline returns the line (chuck-norris tests without trailing newline pass)
- stderr shown in test-game `output`, not judged
- CG compiles WITHOUT optimization: `#ifdef __OPTIMIZE__` (CPP) is false on CG (roller-coaster test-game 2026-10-21, journal `referee.roller-coaster.-.1792589750622861841`). Local `agent/tool/lang/haskell/build.sh` uses `ghc -O` (2026-10-21): local timing and memory are optimistic, measure with plain `ghc` (-O0) before trusting a slow-ish solution
- CG speed at -O0 equals local -O0 (same program 1508 ms CG vs 1459 ms local)
- local toolchain: nixpkgs 18.09 `haskell.packages.ghc843.ghcWithPackages`, Haskell Platform subset (split, vector, containers, mtl, text, parsec, regex-*, unordered-containers, random); which of these CG ships is unverified
