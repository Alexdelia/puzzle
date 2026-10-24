# Perl env on CG

Verified 2026-10-19 with test games on chuck-norris (stderr probe):

- `$^V` = `v5.34.0`, `$]` = `5.034000`
- source saved as `/tmp/Answer.pl` (`$0`), run as a perl script: `#!perl -nl` first line switches are honored (verified: `-n` loop and `-l` chomp/`$\` on CG and locally, 9/9 chuck-norris). Other switches (`-p`, `-a`, `-E`, `-0`) likely work the same way, unverified
- `x86_64-linux-gnu-thread-multi`, ivsize 8 (64-bit ints), nvsize 8 (no long double), `2**64` prints `1.84467440737096e+19`
- non-core modules present on CG (Ubuntu/Debian packages), all loadable locally too: `List::MoreUtils`, `List::Compare`, `Array::Utils`, `Set::Scalar`, `String::Util`, `Tree` (`Tree::Fast`, `Tree::Binary2`), `Regexp::Assemble`, `Algorithm::Diff`, `Algorithm::Merge`, `Text::Unidecode`, `Unicode::LineBreak`, `Unicode::UTF8`, `DateTime`, `Moose`, `Try::Tiny`, `Capture::Tiny`, `Ref::Util`, `PadWalker`, `Class::XSAccessor`, `XML::LibXML`, `enum`
- absent on CG: `JSON` (use core `JSON::PP`), `Text::CSV`, `Math::BigInt::GMP`; core `List::Util` (sum, max, min, reduce, uniq, pairs...) is the cheap one
- `@INC` differs (CG `/etc/perl /usr/share/perl5 ...`, local nix store + `PERL5LIB` set by the withPackages wrapper): `$ENV{PERL5LIB}` exists locally only
- stderr shows in test-games `output` but is not judged
- local launcher startup ~8 ms (wrapper script around perl)
- raw NUL in source is whitespace/ignored between tokens on CG perl 5.34 (`print 1\0+2` -> 3, `my\0$x` -> 1, trailing/leading NUL ok; `q\0ab\0` uses NUL as quote delimiter), verified locally with `$CG_PERL`. CG strips NULs on submit, so this never saves bytes: the stored (stripped) program must stand alone; probe auto-commit compares the CG row length with the LOCAL `count.sh` length, so padding cannot fake a gain either.
