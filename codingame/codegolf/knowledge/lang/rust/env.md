# Rust on CG: environment

Verified with test games (2026-10-19):

- `rustc 1.90.0 (1159e78c4 2025-09-14)`, LLVM 20.1.8, from `/opt/coderunner/rust/target/.rustc_info.json`
- edition 2024: `$e:expr` macro fragment matches `_` (2024-only behaviour). 2024 keywords (`gen`) are reserved, 2024 prelude applies
- release build: `cfg!(debug_assertions)` false, so integer overflow wraps silently instead of panicking
- source compiled from `/tmp/answer.rs`, binary `/tmp/Answer`, `available_parallelism` = 1, `RUST_BACKTRACE=1`, `HOME=/tmp`, stdout unbuffered via `LD_PRELOAD` libstdbuf
- crates prebuilt by cargo (`cargo build --release` of `/opt/coderunner/rust/Cargo.toml`, rlibs in `target/release/deps`): `chrono =0.4.41`, `itertools =0.14.0`, `libc =0.2.175`, `rand =0.9.2` (feature `small_rng`), `rand_core =0.9.3`, `regex =1.11.2`, `time =0.3.43`. Usable by path without `extern crate` (e.g. `rand_core::...` resolves). Transitive deps (`either`, `memchr`, `num_traits`, `aho_corasick`, ...) rlibs also present, reachability unverified
- a failing compile returns only ONE rustc error in `error.message` (with line), not the full list
- local toolchain (`agent/tool/lang/rust/`) has NO crates yet: code using them compiles on CG only
