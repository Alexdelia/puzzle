# F# environment on CG

CG verified (chuck-norris test-games, 2026-10-19), local toolchain matches:

- .NET runtime 8.0.8 (SDK 8.0.401), F# 8 language, FSharp.Core assembly 8.0.0.0, file version 8.4.24.35102 (= FSharp.Core 8.0.400)
- Debian 11 container, entry assembly `/tmp/Answer.dll`, culture `en-US`
- Debug build: `DebuggableAttribute` with JIT optimizer disabled, tailcalls off (mutual `let rec` recursion of depth 1000 gives 2002 stack frames), `DEBUG` symbol not defined (`#if DEBUG` is false)
- deep recursion therefore costs stack: a tail-recursive golf loop of millions of steps across functions may overflow; self tail recursion inside one `let rec` still compiles to a loop (unverified on CG)
- top-level implicit entry point works, `stdin.ReadLine()` fine on input without final newline
- startup: ~70-110 ms wall per run locally (`dotnet Answer.dll`)

Local build: `agent/tool/lang/fsharp/build.sh` calls `fsc.dll` directly with `--debug:portable --optimize- --tailcalls-`.
