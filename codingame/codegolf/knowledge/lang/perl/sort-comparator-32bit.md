# sort{$a-$b}: the comparator is truncated to 32-bit

Verified 2026-10-25 with CG Perl 5.34 and host Perl 5.42 (network-cabling).

`sort{$a-$b}` beats `$a<=>$b` by 2 bytes, but perl reads the block's return value as a signed 32-bit int. A pair whose difference leaves [-2^31+1, 2^31-1] flips sign and the comparator becomes inconsistent: `sort` then returns an arbitrary permutation (no error, no warning):

    perl -e '@a=(0,2147483648,-2147483648);print"@a\n";@b=sort{$a-$b}@a;print"@b\n"'
    0 2147483648 -2147483648
    0 2147483648 -2147483648        (still unsorted)

    perl -e '@a=(0,2147483647,-2147483647);@b=sort{$a-$b}@a;print"@b\n"'
    -2147483647 0 2147483647

`$a<=>$b` is exact; `$a-$b` is safe only when every pair difference is known to fit 32-bit signed. For values bounded by `+-2^30` the largest possible span is exactly 2^31, so `+-2^30` corners break it while `2^30` and `1-2^30` are safe. Same for a `$a-$b` hidden inside a `sort` of any other expression (line sort, key sort).
