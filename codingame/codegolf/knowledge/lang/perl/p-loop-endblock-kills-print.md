# `-p` + `}{` cancels the per-line auto-print

Verified locally on CG's Perl 5.34 (`$CG_PERL/bin/perl`), 2026-10-25:

    printf 'a\nb\n' | perl -pe 'print "L:";}{print "END\n"'
    -> L:L:END

The `}{` in the body splits perl's generated `while (<>) { CODE } continue { print }`
so that the `continue`/print block is neutralized: `CODE` before `}{` still runs for
every line, code after `}{` runs once after EOF, and **nothing is auto-printed**.
With `-ple`, `$\` is still "\n" (explicit prints get a newline) and input is chomped;
`-ne` + `}{` is the ordinary END-block idiom.

Use when `-p`'s read/`-l` behavior is wanted but only some lines may print. Cost:
`#!perl -pl` (11) + `}{` (2) = 13 vs `#!perl -ln` = 11, so it only pays when the END
block is needed too or when the auto-print would otherwise be in the way.

Related: [nl-ternary-endblock](nl-ternary-endblock.md) (`-nl` + `}{` final block).
