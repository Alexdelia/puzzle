# Dart runtime on CG

CG verified: yes (chuck-norris test-games, 2026-10-19).

- `Platform.version`: `3.9.4 (stable) (Tue Sep 30 12:08:50 2025 -0700) on "linux_x64"`, same build locally (nixpkgs `ee09932c`)
- source file is `Answer.dart`, run from source by the JIT VM: `Platform.executableArguments` empty, asserts off, `dart.vm.product` false
- null safety is mandatory (Dart 3): `stdin.readLineSync()!` needs the `!`
- stderr is shown in the test-game `output` but not judged
- local: `agent/tool/lang/dart/build.sh` precompiles a VM kernel snapshot `Answer.dill` (`dart --snapshot-kind=kernel`), same flags as CG. Only drift: `Platform.script` ends in `Answer.dill` locally, `Answer.dart` on CG (matters only for code that reads its own source)
- do not use `dart compile kernel` for local runs: it bakes `dart.vm.product=true` into constants, CG has false
- startup: local launcher ~65-95 ms wall (snapshot, no parse); CG parses source at each run, so expect more there
