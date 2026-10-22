# Spawning an interpreter from F#

Verified on CG 2026-10-22 (temperature-code-golf F# tests 1/3, the-descent-codesize test 4 game 910937146):

- `System.Diagnostics.Process.Start("perl","-e <code>")` at top level (implicit entry point, both io and interactive) runs the child on the same stdin/stdout: .NET Core defaults to `UseShellExecute=false`, no redirection, PATH lookup of the file name. The non-unit result only warns (FS0020).
- interactive: append `.WaitForExit()` (CG kills orphan children at main exit, `knowledge/lang/_shared/orphan-child-kill.md`); io passes without it.
- two overloads: `Start(file, string)` and `Start(file, seq<string>)`; an F# list literal works for the second: `Process.Start("lua",["-e";"""<code>"""])`. List form costs `[`, `;`, `]` + one literal per argument; string form costs .NET argv quoting.
- .NET argv string rules (Unix `ParseArgumentsIntoList`, verified by round trip locally): only space and tab separate arguments (a newline does not), `"` toggles quoting anywhere in a word, `\"` is a literal quote, backslashes are literal unless they precede a `"` (then 2n -> n). So a core without space/tab/`"` goes raw: `"-e <core>"`; otherwise wrap it in `"..."` and escape inner quotes.
- F# literals: regular `"..."` keeps unknown escapes (`\d`, `\w`, `\s` stay two chars), only `\` before `n t b r a f v \ " ' 0-9 x u U` or newline needs doubling; verbatim `@"..."` only doubles `"`; triple `"""..."""` escapes nothing (content must not contain `"""` or end with `"`).
- F# box interpreters (probe test 1, `command -v`): PATH=/opt/coderunner/dotnet/sdk:/bin:/usr/bin:..., `/usr/bin/perl` 5.34, `/usr/bin/python3`, `/bin/lua` 5.4.4, `/bin/php` 7.3.9, `/bin/bash`; ruby only `/usr/local/bin/ruby` (bare `ruby` fails); no node/nodejs.
- `php -r` in a spawned child prints warnings on stdout: some cores need `-ddisplay_errors=` (local referee catches it, temperature test 06).

Generators: `agent/wrap/fsharp/` (`wrap.sh <puzzle> --host fsharp`).
