# VB.NET env on CG

Probes 2026-10-19 on `chuck-norris-codesize`, CG `VB.NET`:

- runtime `.NET 8.0.8` (SDK 8.0.401), VB 16.9, Microsoft.VisualBasic.Core 13.0.0.0 (file 13.0.824.36612), Debian 11, 1 cpu
- compiled as an SDK project: root namespace `Answer` (a `Module Solution` is `Answer.Solution`), assembly `Answer`, run as `/tmp/Answer.exe`, cwd `/tmp`
- implicit project imports work without `Imports`: `System` (Console, Convert), `System.Collections` (ArrayList), `System.Collections.Generic` (List), `System.Diagnostics` (Debug), `System.Linq` (Enumerable, `.Select` on strings), `System.Threading.Tasks` (Task), `System.Xml.Linq` (XML literals `<a/>`), `Microsoft.VisualBasic` (Chr, AscW)
- NOT imported: `System.Text` (StringBuilder, CG error BC30002). Locally `System.Data` (DataTable) and `System.Text.RegularExpressions` (Regex) also fail; on CG only the first compile error is shown, so those two are verified locally only
- Option Strict Off: `Dim s As String = 5` and late binding `o.Length` on Object compile
- Option Explicit On: undeclared `q = 5` fails BC30451 (put `Option Explicit Off` on line 1 to drop `Dim`)
- Option Infer On (`Dim f = ""` is String), Option Compare Binary (`"a" = "A"` false)
- integer overflow checked: `Integer.MaxValue + 1` throws OverflowException
- defines: `NET8_0`, `NET`, `NET8_0_OR_GREATER`, `NET5_0_OR_GREATER`, `NETCOREAPP3_1_OR_GREATER`, `NETCOREAPP`, `PLATFORM="AnyCPU"`, `TARGET="exe"`, `_MyType="Empty"`, `VBC_VER`; no `DEBUG`, `TRACE`, `CONFIG`
- JIT optimizer disabled (debug build), culture en-US, utf-8 out
- CG compile errors report only the first error, with its line
- launcher local startup ~75 ms
