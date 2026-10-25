# A Ruby cell's shebang interpreter is forced to ruby; only its ruby flags pass through

Verified on CG 2026-10-25 with network-cabling-codesize test games (Ruby language):

- `#!perl -0p\n$_=42` -> `/usr/local/bin/ruby: no Ruby script found in input (LoadError)`:
  the `perl` name is not used to run the file, so a Ruby row cannot be a Perl file.
- `#!ruby -0p\n$_=42` -> prints `42`: `-0` slurp plus `-p` auto-print are honored as usual.

Consequence for cross-language cores: reaching another interpreter from a Ruby cell needs an
`exec`/`system` wrapper (`exec"perl -0pe'<core>'"` = 17 B over the core, longer with quotes), not a
shebang.  Corollary: copying another language's leaderboard row into the Ruby slot is impossible.

Also verified 2026-10-25 (local launcher = `$CG_RUBY/bin/ruby <file>`, same as
`agent/tool/lang/ruby/build.sh`): `-e` code in the shebang is **not** honored.
`#!ruby -0anep42`, `#!ruby -e'p 1'`, `#!ruby -e p 3` all produce no output; only the
plain flags (-0 -a -n -p -l -F) are applied. An earlier journal claim that
`#!ruby -0ane'CODE'` works is wrong; put the code on line 2.
