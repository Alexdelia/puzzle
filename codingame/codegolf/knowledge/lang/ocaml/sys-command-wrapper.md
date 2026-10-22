# Sys.command wrapper

Verified on CG (dont-panic row 96, 2026-10-22): `let _=Sys.command"perl -anE'...'"` runs the command through `/bin/sh`, interactive stdin/stdout are inherited, the OCaml process waits for the child (no orphan kill). Wrapper overhead 19 bytes + core. OCaml string: only `"` and `\` need escaping, perl `$`/`{`/`@` are literal.
