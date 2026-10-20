# Perl

- [env](env.md): Perl 5.34.0, 64-bit ints, CG non-core modules (List::MoreUtils, Set::Scalar, ...) mirrored locally, `#!perl -nl` shebang switches honored, verified
- [literal-newline](literal-newline.md): raw newline in regex/string instead of `\n`, saves 1 byte, verified
- [sprintf-vector-flag](sprintf-vector-flag.md): `sprintf"%v07b",$&` = 7-bit binary of a char, 2 bytes under `"%07b",ord$&`, verified
- [autoflush](autoflush.md): interactive puzzles need `$|=1;` or turn 1 times out (stdout block-buffered), verified on CG
- [clear-underscore-autosplit](clear-underscore-autosplit.md): `-ap` + `($x,$y,$_)=@F` reads header and empties `$_` for `-p` output, `s/^/X/` prepend, verified
- [integer-division](integer-division.md): exact IV `/`, `-Minteger` reaches eval strings, float `/20` breaks above 2^53, verified
- [slurp-regex-columns](slurp-regex-columns.md): `-0p` slurp, `/ (.+)/g` column extraction, `$_=answer` as output, `sort{$a-$b}`, local-tested
- [shebang-no-M](shebang-no-M.md): `-M` on the `#!` line dies "Too late"; use `use integer;` or exact division `($r-$r%20)/20`, verified
- [sprintf-vector-join](sprintf-vector-join.md): `sprintf"%*v07b","",$_` whole-string 7-bit binary with empty join (needs -l), `chop` vs `s/.//` for a trailing separator, verified
- [shebang-no-E](shebang-no-E.md): `#!perl -anE` refused, use `-anl` + print; hash slice with undef index harmless, array slice clobbers [0], verified
- [shebang-no-E](shebang-no-E.md): `-E` on `#!` line is fatal; use `print` with `-nl`, verified locally
- [p-loop-header-first-iteration](p-loop-header-first-iteration.md): `-alp` loop, init from header on `$#F`, `$|=(list assign)` autoflush for free, `("",S,N)[$v]`, verified on CG
- [scalar-list-assign-autoflush](scalar-list-assign-autoflush.md): `$|=($a,$b)=<>=~/\d+/g;` merges autoflush into the header read; `for<>=~/./g` chars of next line, verified
- [anon-hash-lookup](anon-hash-lookup.md): `-0p` + `$_={k,v,...}->{$_+0}` hardcode table keyed on first number, verified locally
- [for-growing-array](for-growing-array.md): growing-list flood, ref marks, `>>2`, `=~O`, `/(.+) (.+)/g`, verified
