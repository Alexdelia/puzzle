# Rust

- [env](env.md): rustc 1.90.0, edition 2024, release (overflow wraps), crates chrono/itertools/libc/rand 0.9/regex/time on CG but not locally, verified
- [spawn-wrapper](spawn-wrapper.md): `unsafe extern{safe fn system(_:&str);}` + NUL-terminated literal = 62 B shell wrapper, Command forms, interpreters in the box, verified
