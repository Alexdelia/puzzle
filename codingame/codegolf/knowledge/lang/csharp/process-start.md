# Spawning an interpreter from C#

Verified on CG 2026-10-22 (temperature-code-golf C# tests 1/3, the-descent-codesize test 4 game 910946718):

- top-level statement `System.Diagnostics.Process.Start("perl","-e <code>");` runs the child on the same stdin/stdout (.NET Core `UseShellExecute=false` default, no redirection, PATH lookup). Full name `System.Diagnostics.` (19) beats `using System.Diagnostics;` (25) for one call.
- interactive: top-level statements work too (no `Player` class needed); append `.WaitForExit()` (CG kills orphan children at main exit, `knowledge/lang/_shared/orphan-child-kill.md`). io passes without it.
- overloads `Start(file,string)` and `Start(file,IEnumerable<string>)`: a C# 12 collection expression `["-e",<lit>]` binds to the second (compiles on CG). String form uses .NET argv quoting (same rules as F#, `knowledge/lang/fsharp/process-start.md`): space/tab split, `"` toggles, `\"` literal quote.
- literals: regular `"..."` (escape `\`, `"`, newline), verbatim `@"..."` (double `"`), C# 11 raw `"""..."""` single-line (no newline, must not start/end with `"`) and multi-line `"""\n...\n"""` (closing at column 0 keeps content as is): both raw forms compile and run on CG.
- C# box (probe test 1, `command -v`): PATH=/opt/coderunner/dotnet/sdk:/bin:/usr/bin:..., `/usr/bin/perl` 5.34, `/usr/bin/python3`, `/usr/bin/python`, `/bin/lua` 5.4.4, `/bin/php` 7.3.9, `/bin/bash`; ruby only `/usr/local/bin/ruby`; no node/nodejs.
- bash cores using `. $0` (re-source own file) fail in any `-c` wrapper: `$0` is the interpreter name.

Generators: `agent/wrap/csharp/` (`wrap.sh <puzzle> --host csharp`).
