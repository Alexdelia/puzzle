# Rust: spawning an interpreter

Verified 2026-10-22 (CG test games, journal `wrap.-.rust.1792694203824252295`): CG Rust box runs perl, python3, bash, lua, php on PATH and `/usr/local/bin/ruby` (ruby off PATH, same image as the other boxes).

Shortest forms (rustc 1.90, edition 2024):

- `unsafe extern{safe fn system(_:&str);}fn main(){system("<shell command>\0");}` = 62 + command. Edition 2024 requires `unsafe extern`; `extern` without ABI string defaults to "C"; `safe fn` drops the `unsafe{}` at call site. `&str` is not FFI-safe (warning only) but its pointer lands in the first argument register, so a NUL-terminated literal works. `system` runs `/bin/sh` = dash and waits: good for io and interactive
- a raw NUL byte (U+0000) inside the string literal instead of `\0` saves 1: rustc accepts it, CG test games accept it (temperature-code-golf test 2, the-descent-codesize game 910946330). Unverified: whether a submitted solution keeps the NUL byte (submit.sh confirms)
- `extern"C"` without space after `extern` is a 2021+ reserved-prefix error: `extern "C"`, or better no ABI string
- `fn main(){std::process::Command::new("perl").args(["-e",<lit>]).status();}` = 67 + interpreter + flags + literal: `.status()` inherits stdin/stdout and waits, no shell quoting (wins when the core has `'`; `system` costs 62 + `perl -e''`). io only: `.spawn()` (-1 B) also passes on CG (child outlives main on io)
- Rust string literals take raw newlines; `r#"..."#` avoids escaping `\` and `"`; a bare CR in a literal is a compile error
