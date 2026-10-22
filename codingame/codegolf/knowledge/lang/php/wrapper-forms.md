# PHP wrapper forms and CG PHP box interpreters

Verified on CG 2026-10-22 (temperature-code-golf, PHP test games, journal `agent/state/journal/wrap.-.php.1792694203663544996.md`):

- CG PHP box PATH=`/bin:/usr/bin:/usr/GNUstep/Local/Tools:/usr/GNUstep/System/Tools:/opt/coderunner/groovy/bin`: `/usr/bin/python3`, `/bin/lua` (5.4.4), perl, bash on PATH; ruby only at `/usr/local/bin/ruby` (off PATH); no node/nodejs
- io: `` <?=`cmd`; `` (shell_exec, output printed at exit) is 7 bytes under `<?system('cmd');` and needs no PHP escape of `'`; only `` ` ``, `\` before an escape letter, and `$` before a name char or `{` need a backslash. `;` mandatory (`<?=expr` at EOF is a parse error). Passed CG with python3 -c, lua -e, /usr/local/bin/ruby -e
- interactive: `<?system('cmd');` (child inherits stdin, system waits, output relayed per line), see [system-perl-core](system-perl-core.md)
- PHP single-quoted string: `\` only needs doubling before `\`, `'` or at string end
- nowdoc `<?system(<<<'P'\ncmd\nP);` (7.3 flexible close: `P);` on one line) avoids all PHP escaping, +7 over `<?system('...');`; each `'` of cmd saves 1 there, a shell `'\''` saves 4, so it wins with many quotes (local-verified)
- shell side: `perl -e"$(cat<<\E\ncore\nE\n)"` passes the core without shell quoting (dash, local-verified), +14 over `perl -e'core'`; under system('') each core `'` then costs 2 instead of 8 (`\'\\\'\'`), so it wins at 3+ `'`
