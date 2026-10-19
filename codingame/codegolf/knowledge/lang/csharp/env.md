# C# env on CG

Probes 2026-10-19 on `chuck-norris-codesize`, CG `C#`:

- runtime `.NET 8.0.8` (SDK 8.0.401), C# 12, Debian 11, 1 cpu
- top-level statements accepted: no `class`/`Main` needed
- NO implicit global usings: `Console` without `using System;` fails `CS0103`. Every namespace costs its `using` (or a full name like `System.Console`)
- no preprocessor symbols: `DEBUG`, `TRACE` undefined
- unsafe code allowed (`-unsafe`)
- JIT optimizer disabled (debug build): slow loops cost more than usual
- arithmetic unchecked: `int.MaxValue+1` wraps
- culture en-US (`1.5` prints `1.5`), stdin/stdout utf-8
- entry assembly `Answer`, run as `/tmp/Answer.exe`, cwd `/tmp`
- launcher local startup ~50-110 ms
