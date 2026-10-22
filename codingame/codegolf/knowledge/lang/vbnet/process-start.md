# Spawning an interpreter from VB.NET

Verified on CG 2026-10-22 (temperature-code-golf VB.NET test 3 for perl/python3/sh/lua/php/`/usr/local/bin/ruby`, the-descent-codesize test 4 game 910946541 interactive):

- shortest frame: `Module A` / `Sub Main` / `<call>` / `End Sub` / `End Module`, newline-separated (37 B around the call). Module name is free (`A` works on io and interactive puzzles, no `Solution`/`Player` needed). Putting `Module A:Sub Main:...` on one line with `:` does not compile (local)
- `Process.Start("perl","-e <code>")`: `System.Diagnostics` is a default project import, so no qualifier. Child shares stdin/stdout (.NET Core `UseShellExecute=false`, no redirection), PATH lookup. Statement call discards the result
- interactive: `.WaitForExit` (no parens, +12) is required, CG kills orphan children at main exit (`knowledge/lang/_shared/orphan-child-kill.md`). io passes without it
- second overload: array literal `Process.Start("lua",{"-e","<code>"})` (IEnumerable(Of String)), avoids .NET argv quoting; string overload follows .NET argv rules (see `knowledge/lang/fsharp/process-start.md`)
- VB string literal: only `"` is escaped (doubled `""`), no backslash escapes, raw newlines allowed inside the literal (VB 14+)
- `Shell("perl -e ...",,1)` (Microsoft.VisualBasic.Interaction) is shorter but throws PlatformNotSupportedException "Method requires System.Windows.Forms" on Linux (local .NET 8, same runtime as CG)
- box: perl, python3, sh/bash, lua, php on PATH; ruby only at `/usr/local/bin/ruby` (bare `ruby` untested); `node` missing (Win32Exception (2) No such file or directory)

Generators: `agent/wrap/vbnet/` (`wrap.sh <puzzle> --host vbnet`).
