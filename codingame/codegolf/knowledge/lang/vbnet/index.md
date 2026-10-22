# VB.NET

- [env](env.md): .NET 8.0.8 VB 16.9, root namespace Answer, project imports (System, Generic, Linq, Microsoft.VisualBasic...) implicit but not System.Text, Strict Off, Explicit On, Infer On, overflow checked, no DEBUG/TRACE, unoptimized JIT, verified
- [process-start](process-start.md): `Module A`/`Sub Main` + `Process.Start("perl","-e ...")` wrapper (37 B frame, +12 `.WaitForExit` interactive), Shell() unsupported on Linux, box has perl python3 sh lua php, ruby at /usr/local/bin, no node, verified
