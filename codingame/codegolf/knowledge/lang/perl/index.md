# Perl

- [env](env.md): Perl 5.34.0, 64-bit ints, CG non-core modules (List::MoreUtils, Set::Scalar, ...) mirrored locally, `#!perl -nl` shebang switches honored, verified
- [literal-newline](literal-newline.md): raw newline in regex/string instead of `\n`, saves 1 byte, verified
- [sprintf-vector-flag](sprintf-vector-flag.md): `sprintf"%v07b",$&` = 7-bit binary of a char, 2 bytes under `"%07b",ord$&`, verified
- [autoflush](autoflush.md): interactive puzzles need `$|=1;` or turn 1 times out (stdout block-buffered), verified on CG
