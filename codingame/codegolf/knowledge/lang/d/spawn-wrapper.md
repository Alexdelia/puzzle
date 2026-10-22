# Spawn another interpreter from D

Verified on CG 2026-10-22 (D, DMD 2.099), generators in `agent/wrap/d/`:

- `import std;void main(){`<sh command>`.spawnShell.wait;}` = command + 43. `spawnShell` runs `/bin/sh -c` (phobos `nativeShell`, dash on CG), child inherits stdin/stdout/stderr. Interactive works with plain `void main` (the-descent-codesize test games 910946414, 910946427): no Player/Solution entry point needed.
- argv form, no shell quoting: `import std;void main(){[`perl`,`-e`,`<core>`].spawnProcess.wait;}` (PATH lookup works: `lua`, `/usr/local/bin/ruby`).
- io: `.wait` can go (-5), the child still answers (temperature-code-golf test 1, perl/python/bash/ruby). Interactive needs it (orphan kill, `knowledge/lang/_shared/orphan-child-kill.md`).
- string literal: backtick WYSIWYG string takes the core raw, newlines included, no escape; core with a backtick: `r"..."` if no `"`, else `"..."` escaping `\` and `"` only (literal newlines allowed).
- D box interpreters (`command -v` probe, temperature-code-golf test 1): perl 5.34, python3, python, `/bin/lua`, `/bin/php`, `/bin/bash`; ruby only `/usr/local/bin/ruby`; no node/nodejs.
