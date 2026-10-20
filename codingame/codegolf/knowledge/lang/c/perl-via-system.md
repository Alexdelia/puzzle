# perl core inside system()

Verified network-cabling (local 21/21, CG accepted 128, 2026-10-20): `main(){system("perl -0nE'...'");}`. `perl` runs on CG from C's `system`, single quotes need no escape, `$` and `@` pass through; avoid `\S` (needs `\\S`), use `.`.

- `-0n` slurps stdin into `$_`; header `N` as number is `$_` (`$_/2`)
- perl `sort{$a-$b}` breaks when a difference reaches 2^31 (comparator result cut to int, local perl 5.42); `sort{$a>$b}` unreliable; use `sort{$a<=>$b}`; wrap as `sub f{sort{$a<=>$b}@_}` when sorting twice (`@x=f/(.+) /g` parses fine)
- `/(.+) /g` yields column 1, `/ (.+)/g` column 2 of `x y` lines
- sorted array + `abs$_-$y[@y/2]for@y` beats pair-ends loop by 6 bytes in perl (120 vs 126)
