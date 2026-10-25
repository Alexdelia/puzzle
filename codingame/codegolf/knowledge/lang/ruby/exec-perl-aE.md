# Ruby -> Perl one-liner: `-aE` wrapper is 16 B + body source

Verified 2026-10-25 locally (Ruby 3.1.2, perl 5.42; Ruby box CG-verified for perl earlier).

- `perl -a` alone implies `-n`: `echo 'a b' | perl -aE 'say scalar @F'` prints `2`. So
  `-aE` gives autosplit + a per-line loop + `say` with **no `-p`, no `-l`**.
- Ruby wrapper costs (`exec"perl FLAGS'BODY'"`, flags attached to the quoted body so the shell
  produces one `-FLAGSBODY` word that perl reads as `-e BODY`):
  - `-aE` (16 B over the body source), `-alp` (17), `-E` (15, but then no line loop),
    `-apE` (17), `exec"bash -c 'CORE'"` (16).
- Ruby double-quoted string escapes: every `\` and every `"` in the body costs +1 source byte
  (`\\D`, `\"@m\"`). `$`, `@`, `|`, `{`, `}` are literal. With `-aE` (no `-l`) the input
  line keeps its `\n`, so `/\D/g` also matches the space+newline of the init line, both
  writable (`${" "}`, `${"\n"}`), and `say"@m"` prints. The body is 79 B (same shape as the
  `-alp` one, `say` instead of `$_=`, `\D` still needed), so the row is **98 B**
  (16 + 79 + 3 escapes); the `-alp` route is 99.
- Quote-free print (`say@m`, 5) needs `$,=$"` (+5, once in the guard) or a `$"` element in
  `@m` (+4 plus a `2*/U|D/` index +2): all net +3 or worse. `say@m` alone joins with undef.
- The shell strips backslashes in unquoted words: `-F\D\K` reaches perl as `-FDK`. Quoting or
  doubling the pattern in the Ruby source makes `knowledge/lang/perl/F-nondigit-K-split.md`
  (76 B body) net +2..+5, so it does not help Ruby.
- Ruby 95 (= top, Icebox/cedricdd) is a real byte row; with this wrapper it needs a 75 B body
  (`\D` + 2 quotes) / 76-78 B with fewer escapes, or a 79 B bash core in the 16 B bash wrapper.
