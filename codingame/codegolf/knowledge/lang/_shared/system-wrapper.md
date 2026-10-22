# system() wrapper

Verified 2026-10-19 on `don't-panic` test 7 (interactive, Python3): `import os;os.system('''bash -c '<core>' ''')` runs `bash` on CG, the child inherits stdin/stdout, interactive turns work, stderr of the child shows in the frame log.

So any lang with a shell call costs wrapper + shortest core in a shell-reachable interpreter (bash; perl/ruby availability unverified). Leaderboard evidence (unverified, arithmetic only): `don't-panic` C top 72 = `main(){system("");}` (19) + bash top 53.

Unverified: which interpreters are on PATH in the CG box besides bash, whether every lang runtime allows process spawn.

Verified on CG 2026-10-20 (shadows-of-the-knight-episode-1, C): `system()` runs `/bin/sh -> dash` (not bash), so a bash core needs `bash -c '...'` (+10 bytes). C wrapper total = `main(){system("bash -c '<core>'");}` = core + 29. Core with `"`, `\` or newline needs C escapes; `;` instead of newline costs nothing. `sed 3q>&2` inside the child skips 3 input lines interactively on CG (1 byte under `head -3>&2`).

Trap (verified 2026-10-20, super-computer): CG `system()`/`exec`/`os.system`/backticks children run `/bin/sh` = dash, so a core that only works in bash fails on CG. Example: `read;` (no variable) in dash prints `read: arg count` and consumes no line; `read a;` works. Since 2026-10-20 every local launcher runs under bwrap with `/bin/sh` -> dash 0.5.11 (`agent/tool/lang/cache.sh`), so the local referee catches it; outside the referee check with `$CG_DASH/bin/dash -c '<core>' < test.in`.

Verified 2026-10-20 (mars-lander-episode-3, Bash test game 910407501): the CG Bash box has `/usr/bin/python3` (3.11.5), `/usr/bin/perl`, `/bin/php` on PATH (no node). Perl `exec"/usr/bin/python3",...` and Ruby `exec"/usr/bin/python3","-c",DATA.read` also run Python on CG (submits 100%/50% as predicted). Local perl/ruby sandboxes have empty PATH and no python3: such wrappers fail locally only.

Verified 2026-10-20 (blunder-episode-1, non-zero partial scores): `/usr/bin/perl` with inherited stdin is reachable from Ruby `exec"/usr/bin/perl","-e",DATA.read` + `__END__`, PHP `passthru("/usr/bin/perl -e ".escapeshellarg(<<<'P' ... P))`, JavaScript `require("child_process").execFileSync("/usr/bin/perl",["-e",String.raw`...`],{stdio:"inherit"})`, Lua `os.execute("/usr/bin/perl -e '"..[==[...]==].."'")`. Handy to run one perl probe core in many fresh languages.

Trap (verified 2026-10-21, the-labyrinth, TypeScript test game 910610308): the CG TypeScript box `/usr/bin/perl` is v5.34.0, so a spawned perl core has the 5.34 variable-length lookbehind bug (`"X.T"=~/(?<=X|X.{2})T/` matches on CG, not on perl 5.42). Local launchers of every lang except bash and perl inherit the host PATH, so a spawned `perl` there is host perl 5.42 and the local referee passes code CG rejects (TS 262/258 cores with `(?<=X.|X.{30}.)`: local 16/16, CG rejected). Until the launchers pin `$CG_PERL`, check spawned perl cores with `PATH=$CG_PERL/bin:$PATH <referee>/run.sh <file> --lang <lang>`.

Fixed 2026-10-21: every launcher now provides CG perl 5.34 at `/usr/bin/perl` and first on PATH, and CG python 3.11.5 at `/usr/bin/python3` (`agent/tool/lang/cache.sh`), so the TS 258 labyrinth core fails locally (0/8) as on CG. Availability per CG box still differs (no ruby in the clojure box): the launcher does not hide host interpreters other than perl/python.

Lua (2026-10-21): the CG Groovy box has `lua` on PATH (music-scores groovy 294 = `ProcessBuilder('lua','-e',...)`, CG 100%). Other boxes unverified: run one test game printing `lua -v` before relying on it in a new language. Every local launcher (bash included) now puts CG lua 5.4.4 on PATH, so the local referee passes a lua wrapper in any language: in a box where CG lacks lua, it fails only on CG.

Haskell box (verified 2026-10-22, `command -v` probe, `knowledge/lang/haskell/system-wrapper.md`): perl, python3, python, lua 5.4.4, php 7.3.9, bash on PATH; no ruby, no node. python3/lua/php `system` wrappers pass on CG.

Python3 box (verified 2026-10-22, temperature-code-golf testcase 1 via `os.system`): `lua` and `php` on PATH, `ruby` not (`sh: 1: ruby: not found`). Details: `knowledge/lang/python/perl-wrapper.md`.

Javascript box (2026-10-22, dwarfs-standing-on-the-shoulders-of-giants-codesize test 1, knowledge/lang/javascript/spawn-interpreters.md): `bash`, `lua`, `php`, `perl`, `python3` on PATH, `ruby` not found.

Lua box (verified 2026-10-22, knowledge/lang/lua/box-interpreters.md): python3 3.11.5, perl 5.34, `/bin/php` 7.3.9, bash 5.1 on PATH; ruby 3.1.2 only at `/usr/local/bin/ruby` (off PATH); no node. Trap: a spawned `php -r` shows warnings on stdout (no `display_errors=stderr` as in the PHP launcher), so a php core relying on silent warnings needs `-ddisplay_errors=`. Local launchers have no php and no `/usr/local/bin/ruby`, so such wrappers fail locally only.

PHP box (verified 2026-10-22, temperature-code-golf PHP test games): python3, `/bin/lua` 5.4.4, perl, bash on PATH, ruby only at `/usr/local/bin/ruby`, no node. Details: `knowledge/lang/php/wrapper-forms.md`.

Verified 2026-10-22 (Ruby, CG test games 910933624/910933386/910933406/910933448, all 100%): the CG Ruby box has `perl`, `python3`, `bash`, `lua` on PATH, reached by `exec` both through sh and through Ruby's own PATH lookup (argv form). `knowledge/lang/ruby/exec-wrapper.md`.

Groovy box (verified 2026-10-22, telephone-numbers-codesize test 1, `knowledge/lang/groovy/spawn-interpreters.md`): python3, php, lua, perl, bash on PATH; ruby 3.1.2 only as `/usr/local/bin/ruby`; no node.

Perl box (verified 2026-10-22, details `knowledge/lang/perl/exec-wrapper.md`): `exec` reaches bash (test game 910933623), lua (910933626), php (temperature io test 1 success), python3. Ruby unverified (game 910933627 timed out silently). The local perl launcher has no php/ruby, so those wrappers pass on CG but fail the local referee.

OCaml box (verified 2026-10-22, telephone-numbers-codesize test 1 probe, `knowledge/lang/ocaml/system-wrapper.md`): python3 3.11, `python` = 2.7.16, lua 5.4.4, php 7.3.9, perl 5.34, bash on PATH; ruby only at `/usr/local/bin/ruby`; no node.

Scala box (2026-10-22, temperature-code-golf test 1, `sys.process.Process`, knowledge/lang/scala/process-tokenizer-quoting.md): `sh`, `perl`, `python3`, `lua`, `php` run; `ruby`, `node` not found.

Dart box (verified 2026-10-22, temperature-code-golf test 1, `knowledge/lang/dart/spawn-interpreters.md`): perl 5.34, python3 3.11.5, bash, lua 5.4.4, php 7.3.9 on PATH; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node. Generated perl/python3/sh/lua/php/ruby wrappers pass on CG.

F# box (verified 2026-10-22, temperature-code-golf F# test 1 `command -v` probe, `knowledge/lang/fsharp/process-start.md`): perl 5.34, python3, lua 5.4.4, php 7.3.9, bash on PATH; ruby only at `/usr/local/bin/ruby`; no node. `Process.Start(...)` wrappers of all six pass on CG.

Clojure box (verified 2026-10-22, `knowledge/lang/clojure/box-interpreters.md`): python3, perl, bash, lua, php on PATH; ruby only at `/usr/local/bin/ruby`; no node.

Swift box (verified 2026-10-22, temperature-code-golf test 1 probe, `knowledge/lang/swift/system-wrapper.md`): python3, perl 5.34, php 7.3.9, lua 5.4.4, bash on PATH; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node. `import Glibc;system(...)` wrappers of all six pass on CG.

Rust box (verified 2026-10-22, temperature-code-golf + the-descent-codesize test games, `knowledge/lang/rust/spawn-wrapper.md`): perl, python3, bash, lua, php on PATH, ruby only at `/usr/local/bin/ruby`; generated wrappers of all six pass on CG.

D box (verified 2026-10-22, temperature-code-golf test 1 `command -v` probe, `knowledge/lang/d/spawn-wrapper.md`): perl 5.34, python3, python, lua, php, bash on PATH; ruby only at `/usr/local/bin/ruby`; no node. `spawnShell`/`spawnProcess` wrappers of all six pass on CG.

Java box (verified 2026-10-22, temperature-code-golf Java test 1 `command -v` probe, `knowledge/lang/java/process-wrapper.md`): same as Kotlin: python3, perl 5.34, php 7.3.9, lua 5.4.4, bash on PATH; ruby only at `/usr/local/bin/ruby`; no node. ProcessBuilder wrappers of all six pass on CG.

VB.NET box (verified 2026-10-22, temperature-code-golf test 3 generated candidates, `knowledge/lang/vbnet/process-start.md`): perl, python3, sh, lua, php run via `Process.Start`; `/usr/local/bin/ruby` runs; no node.

C# box (verified 2026-10-22, temperature-code-golf C# test 1 `command -v` probe, `knowledge/lang/csharp/process-start.md`): perl 5.34, python3, python, lua 5.4.4, php 7.3.9, bash on PATH; ruby only at `/usr/local/bin/ruby`; no node. Generated `Process.Start(...)` wrappers of all six pass on CG.

C box (verified 2026-10-22, temperature-code-golf C test 1 probe, `knowledge/lang/c/system-wrapper.md`): perl 5.34, python3 3.11.5, python, lua 5.4.4, php 7.3.9, bash on PATH; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node. Generated `system` wrappers of python3/lua/php/ruby pass on CG.

ObjectiveC box (verified 2026-10-22, temperature-code-golf test 1 probe, `knowledge/lang/objective-c/system-wrapper.md`): python3, perl 5.34, php 7.3.9, lua 5.4.4, bash on PATH; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node. `main(){system(...)}` wrappers of all six pass on CG.
