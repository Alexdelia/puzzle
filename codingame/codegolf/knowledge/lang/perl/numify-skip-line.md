# `$x=<>+!<>`: read a number and skip the next line, no shebang

`$l=<>+!<>;` (10 bytes) reads line 1 as a number (`"4\n"+...` numifies, newline gone) and consumes line 2 (`!"5\n"` is `""`, adds 0). Left operand of `+` is read first. Beats `$l=<>+0;<>;` (11) and `$l=<>;<>;` + `chomp`.

Why it matters: `/.{$l}/` with `$l="4\n"` never matches (5.34 allows spaces inside quantifier braces, not newlines, `/x` does not help), so an unchomped line can't be a regex quantifier.

With this, a no-shebang `...;print+(LIST),$/for<>` (rows keep their `\n`, `.` skips it) beats `#!perl -nl` + `$.<N?` header dispatch: the shebang line costs 11, `$l||=$_;$.<4?...:` costs 13 more.

Verified: ascii-art perl 73 -> 71, local 13/13 and CG 100% (2026-10-22). Line 2 must be truthy (non-zero, non-empty); a line `0\n` is `"0\n"`, which is true as a string, so any line with a newline works.
