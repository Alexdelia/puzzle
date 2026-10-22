# Spawn an interpreter from Swift

Verified 2026-10-22 (local swiftc 5.3.3 + CG temperature-code-golf Swift test games, journal `agent/state/journal/wrap.-.swift.1792694203931051483.md`).

- form: `import Glibc;system("perl -e'...'")` (wrapper = 24 + shell command). `system` is in Glibc on Linux (unavailable only on Darwin), top-level code, no entry point to name, same form on io and interactive: `system` waits for the child, which inherits stdin/stdout
- `system` runs `/bin/sh` (dash, `knowledge/lang/_shared/system-wrapper.md`): a dash-compatible bash core goes in as is, else `bash -c '...'`
- literals: `"..."` needs `\\`, `\"`, `\n`; raw `#"..."#` keeps `\` and `"` literal (newline `\#n`), invalid if the text holds `"#` or `\#`; raw multiline `#"""` newline text newline `"""#` keeps newlines too (+8)
- CG Swift box (test 1 probe): PATH `/bin:/usr/bin:...`, `/usr/bin/python3`, `/usr/bin/perl` v5.34.0, `/bin/php` 7.3.9, `/bin/lua` 5.4.4, `/bin/bash`; ruby 3.1.2 only at `/usr/local/bin/ruby`; no node/nodejs. Generated perl/python3/sh/lua/php/ruby wrappers pass on CG
