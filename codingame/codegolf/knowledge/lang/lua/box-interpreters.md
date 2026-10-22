# Interpreters in the CG Lua box

Verified 2026-10-22 with Lua test games on temperature-code-golf (`os.execute"..."`, output in stderr):

- `os.execute` runs `/bin/sh` = dash, PATH `/bin:/usr/bin:/usr/GNUstep/Local/Tools:/usr/GNUstep/System/Tools:/opt/coderunner/groovy/bin`
- on PATH: `python3` (3.11.5), `perl` (5.34.0), `php` (`/bin/php`, 7.3.9), `bash` (5.1.16)
- ruby 3.1.2 exists only at `/usr/local/bin/ruby`, off PATH: `ruby -e` = `sh: 1: ruby: not found`, `/usr/local/bin/ruby -e'...'` works (+15 B)
- no node/nodejs anywhere: no JavaScript core from Lua
- `php -r` prints warnings (e.g. `+""` non-numeric) on stdout on CG, unlike the CG PHP launcher (`display_errors=stderr`): temperature test 6 got `\nWarning: ...` and failed. `php -ddisplay_errors= -r'...'` silences them (+18 B), local CG_PHP 7.3.9 behaves the same
- python3, php -r and /usr/local/bin/ruby wrappers of our temperature cores all pass CG test 3

Wrapper generators: `agent/wrap/lua/` (one candidate per Lua string delimiter `"`, `'`, `[=[ ]=]`, and per shell quoting `'` vs `"`).
